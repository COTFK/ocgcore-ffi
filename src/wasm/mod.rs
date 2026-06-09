//! WebAssembly backend via wasm-bindgen and Emscripten.
//!
//! `build.rs` handles compiling `ocgcore` to WebAssembly via Emscripten, then wasm-bindgen
//! loads the JS helper that Emscripten outputs.
//!
//! This backend is the one that will run on the `wasm32-unknown-unknown` target; for other
//! platforms, see [crate::native::NativeBackend].
//!
//! Compared to the native backend, this module contains additional memory management and
//! supporting code to allow data to safely cross the Rust - JS - WebAssembly boundaries.

mod callbacks;
mod memory;

use js_sys::Uint8Array;
use js_sys::futures::JsFuture;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::ffi::CString;
use std::ffi::c_void;
use std::sync::Mutex;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::types::OCG_CardData;
use crate::types::OCG_DataReader;
use crate::types::OCG_DataReaderDone;
use crate::types::OCG_Duel;
use crate::types::OCG_DuelOptions;
use crate::types::OCG_LogHandler;
use crate::types::OCG_NewCardInfo;
use crate::types::OCG_QueryInfo;
use crate::types::OCG_ScriptReader;
use memory::CoreMemoryAllocation;

use callbacks::CALLBACK_REGISTRY;

static OUTPUT_BUFFER_REGISTRY: Lazy<Mutex<HashMap<usize, Vec<u8>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[cfg(target_arch = "wasm32")]
thread_local! {
    pub static WASM_BACKEND: std::cell::RefCell<Option<WasmBackend>> = const {
        std::cell::RefCell::new(None)
    };
}

#[cfg(target_arch = "wasm32")]
pub fn with_backend<R>(f: impl FnOnce(&WasmBackend) -> R) -> R {
    WASM_BACKEND.with(|backend_cell| {
        let backend_ref = backend_cell.borrow();
        let backend = backend_ref
            .as_ref()
            .expect("ocgcore-ffi not initialized: call OCG_Initialize() first");
        f(backend)
    })
}

/// Raw wasm-bindgen FFI - these methods need additional memory management,
/// which the impl block on WasmBackend handles.
#[cfg_attr(feature = "bundled", wasm_bindgen(module = "/ocgcore.js"))]
#[cfg_attr(not(feature = "bundled"), wasm_bindgen(raw_module = "/ocgcore.js"))]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type WasmBackend;

    #[wasm_bindgen(js_name = default)]
    fn _init() -> js_sys::Promise;

    #[wasm_bindgen(method)]
    fn _OCG_GetVersion(this: &WasmBackend, major: u32, minor: u32);

    #[wasm_bindgen(method)]
    fn _OCG_CreateDuel(this: &WasmBackend, duel: u32, options: u32) -> i32;

    #[wasm_bindgen(method)]
    fn _OCG_StartDuel(this: &WasmBackend, duel: u32);

    #[wasm_bindgen(method)]
    fn _OCG_DuelProcess(this: &WasmBackend, duel: u32) -> u32;

    #[wasm_bindgen(method)]
    fn _OCG_DuelGetMessage(this: &WasmBackend, duel: u32, length_ptr: u32) -> u32;

    #[wasm_bindgen(method)]
    fn _OCG_DuelSetResponse(this: &WasmBackend, duel: u32, response_ptr: u32, len: u32);

    #[wasm_bindgen(method)]
    fn _OCG_DuelNewCard(this: &WasmBackend, duel: u32, info: u32);

    #[wasm_bindgen(method)]
    fn _OCG_LoadScript(this: &WasmBackend, duel: u32, buffer: u32, len: u32, name: u32) -> i32;

    #[wasm_bindgen(method)]
    fn _OCG_DestroyDuel(this: &WasmBackend, duel: u32);

    #[wasm_bindgen(method)]
    fn _OCG_DuelQuery(this: &WasmBackend, duel: u32, length_ptr: u32, info_ptr: u32) -> u32;

    #[wasm_bindgen(method)]
    fn _OCG_DuelQueryCount(this: &WasmBackend, duel: u32, team: u8, location: u32) -> u32;

    #[wasm_bindgen(method)]
    fn _OCG_DuelQueryLocation(this: &WasmBackend, duel: u32, length_ptr: u32, info_ptr: u32)
    -> u32;

    #[wasm_bindgen(method)]
    fn _OCG_DuelQueryField(this: &WasmBackend, duel: u32, length_ptr: u32) -> u32;

    // Emscripten helper methods
    #[wasm_bindgen(method, getter, js_name = wasmMemory)]
    fn get_wasm_memory(this: &WasmBackend) -> js_sys::WebAssembly::Memory;

    #[wasm_bindgen(method)]
    fn _malloc(this: &WasmBackend, size: u32) -> u32;

    #[wasm_bindgen(method, js_name = _free)]
    fn free(this: &WasmBackend, ptr: u32);

    #[wasm_bindgen(method, js_name = addFunction)]
    fn add_function(this: &WasmBackend, func: &js_sys::Function, signature: &str) -> u32;
}

impl WasmBackend {
    /// Initializes the backend by calling the default method on the
    /// `ocgcore.js` module.
    pub async fn new() -> Self {
        let promise = _init();
        let ocgcore = JsFuture::from(promise).await.unwrap();
        ocgcore.unchecked_into()
    }

    pub fn OCG_GetVersion(&self, major: &mut i32, minor: &mut i32) {
        let major_alloc = self.allocate_memory(std::mem::size_of::<i32>() as u32);
        let minor_alloc = self.allocate_memory(std::mem::size_of::<i32>() as u32);

        self._OCG_GetVersion(major_alloc.pointer, minor_alloc.pointer);

        let memory = self.get_wasm_memory();
        let buffer: js_sys::ArrayBuffer = memory.buffer().into();
        let view = js_sys::DataView::new(&buffer, 0, buffer.byte_length() as usize);

        *major = view.get_int32_endian(major_alloc.pointer as usize, true);
        *minor = view.get_int32_endian(minor_alloc.pointer as usize, true);
    }

    pub unsafe fn OCG_CreateDuel(
        &self,
        out_ocg_duel: *mut OCG_Duel,
        options_ptr: *const OCG_DuelOptions,
    ) -> i32 {
        if out_ocg_duel.is_null() || options_ptr.is_null() {
            return 1;
        }

        let options = unsafe { &*options_ptr };
        let payload_keys = [
            options.payload1 as usize,
            options.payload2 as usize,
            options.payload3 as usize,
            options.payload4 as usize,
        ];

        let wasm_options_alloc = self.prepare_options_for_wasm(options);
        let handle_storage_ptr = self.allocate_memory(4);
        let status = self._OCG_CreateDuel(handle_storage_ptr.pointer, wasm_options_alloc.pointer);

        // If successful, extract the handle and store payload keys;
        // if not, erase callbacks and let the status code propagate
        if status == 0 {
            unsafe {
                let duel_handle = self.read_u32(handle_storage_ptr.pointer);
                *out_ocg_duel = duel_handle as usize as OCG_Duel;

                let mut callbacks = CALLBACK_REGISTRY.lock().unwrap();
                callbacks
                    .duel_payloads
                    .insert(duel_handle as usize, payload_keys);
            }
        } else {
            self.cleanup_callbacks_for_payloads(payload_keys);
        }

        status
    }

    pub unsafe fn OCG_DestroyDuel(&self, ocg_duel: OCG_Duel) {
        self._OCG_DestroyDuel(ocg_duel as usize as u32);

        let payload_keys = {
            let mut callbacks = CALLBACK_REGISTRY.lock().unwrap();
            callbacks.duel_payloads.remove(&(ocg_duel as usize))
        };

        if let Some(payload_keys) = payload_keys {
            self.cleanup_callbacks_for_payloads(payload_keys);
        }

        let mut output_buffers = OUTPUT_BUFFER_REGISTRY.lock().unwrap();
        let _ = output_buffers.remove(&(ocg_duel as usize));
    }

    pub unsafe fn OCG_DuelNewCard(&self, ocg_duel: OCG_Duel, info_ptr: *const OCG_NewCardInfo) {
        let info_alloc = self.copy_pod_to_wasm(info_ptr);

        self._OCG_DuelNewCard(ocg_duel as usize as u32, info_alloc.pointer);
    }

    pub unsafe fn OCG_StartDuel(&self, ocg_duel: OCG_Duel) {
        self._OCG_StartDuel(ocg_duel as usize as u32);
    }

    pub unsafe fn OCG_DuelProcess(&self, ocg_duel: OCG_Duel) -> i32 {
        self._OCG_DuelProcess(ocg_duel as usize as u32) as i32
    }

    pub unsafe fn OCG_DuelGetMessage(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        self.call_with_len_output(ocg_duel, length, |len_ptr| {
            self._OCG_DuelGetMessage(ocg_duel as usize as u32, len_ptr)
        })
    }

    pub unsafe fn OCG_DuelSetResponse(
        &self,
        ocg_duel: OCG_Duel,
        buffer: *const c_void,
        length: u32,
    ) {
        if buffer.is_null() || length == 0 {
            self._OCG_DuelSetResponse(ocg_duel as usize as u32, 0, 0);
            return;
        }

        let response_bytes =
            unsafe { std::slice::from_raw_parts(buffer.cast::<u8>(), length as usize) };
        let response_alloc = self.allocate_and_copy_bytes(response_bytes);

        self._OCG_DuelSetResponse(ocg_duel as usize as u32, response_alloc.pointer, length);
    }

    pub unsafe fn OCG_LoadScript(
        &self,
        ocg_duel: OCG_Duel,
        buffer: *const std::ffi::c_char,
        length: u32,
        name: *const std::ffi::c_char,
    ) -> i32 {
        // Safely pass 0 to Emscripten if buffer is null so C++ can catch it
        let buffer_alloc = if buffer.is_null() {
            None
        } else {
            let bytes = unsafe { std::slice::from_raw_parts(buffer as *const u8, length as usize) };
            Some(self.allocate_and_copy_bytes(bytes))
        };
        let buffer_ptr = buffer_alloc.as_ref().map(|a| a.pointer).unwrap_or(0);

        let name_alloc = if name.is_null() {
            None
        } else {
            let mut len = 0;

            while len < 4096 && unsafe { *name.add(len) != 0 } {
                len += 1;
            }

            let bytes = unsafe { std::slice::from_raw_parts(name as *const u8, len + 1) };
            Some(self.allocate_and_copy_bytes(bytes))
        };
        let name_ptr = name_alloc.as_ref().map(|a| a.pointer).unwrap_or(0);

        self._OCG_LoadScript(ocg_duel as usize as u32, buffer_ptr, length, name_ptr)
    }

    pub unsafe fn OCG_DuelQueryCount(&self, ocg_duel: OCG_Duel, team: u8, loc: u32) -> u32 {
        self._OCG_DuelQueryCount(ocg_duel as usize as u32, team, loc)
    }

    pub unsafe fn OCG_DuelQuery(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void {
        self.call_query_with_info(
            ocg_duel,
            length,
            info_ptr,
            |duel, len_ptr, info_wasm_ptr| self._OCG_DuelQuery(duel, len_ptr, info_wasm_ptr),
        )
    }

    pub unsafe fn OCG_DuelQueryLocation(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void {
        self.call_query_with_info(
            ocg_duel,
            length,
            info_ptr,
            |duel, len_ptr, info_wasm_ptr| {
                self._OCG_DuelQueryLocation(duel, len_ptr, info_wasm_ptr)
            },
        )
    }

    pub unsafe fn OCG_DuelQueryField(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        self.call_with_len_output(ocg_duel, length, |len_ptr| {
            self._OCG_DuelQueryField(ocg_duel as usize as u32, len_ptr)
        })
    }

    // Helper methods

    /// Allocates `length` bytes of WebAssembly memory and returns a safe [`Drop`] wrapper
    /// around the pointer.
    fn allocate_memory(&'_ self, length: u32) -> CoreMemoryAllocation<'_> {
        let pointer = self._malloc(length);
        CoreMemoryAllocation {
            core: self,
            pointer,
        }
    }

    /// Injects a provided [`js_sys::Function`] into Emscripten's function table and returns its index.
    fn inject_function_into_emscripten<F>(&self, prop: &str, sig: &str, create_cb: F) -> u32
    where
        F: FnOnce() -> js_sys::Function,
    {
        let prop_str = JsValue::from_str(prop);
        let prop_val = js_sys::Reflect::get(self, &prop_str).unwrap_or(JsValue::UNDEFINED);

        // Return cached index if we've already registered this callback type
        if let Some(idx) = prop_val.as_f64() {
            return idx as u32;
        }

        let func = create_cb();
        let idx = self.add_function(&func, sig);
        js_sys::Reflect::set(self, &prop_str, &JsValue::from(idx)).unwrap();
        idx
    }

    /// Returns a `len` bytes long view into the WebAssembly memory, at the specified pointer.
    fn get_memory_view(&self, ptr: u32, len: u32) -> Uint8Array {
        Uint8Array::new_with_byte_offset_and_length(&self.get_wasm_memory().buffer(), ptr, len)
    }

    /// Reads a little-endian u32 from the WebAssembly memory at `ptr`.
    fn read_u32(&self, ptr: u32) -> u32 {
        let buffer: js_sys::ArrayBuffer = self.get_wasm_memory().buffer().into();
        let view = js_sys::DataView::new(&buffer, 0, buffer.byte_length() as usize);
        view.get_uint32_endian(ptr as usize, true)
    }

    /// Copies a plain-old-data struct from Rust memory into Emscripten memory.
    fn copy_pod_to_wasm<T>(&self, value_ptr: *const T) -> CoreMemoryAllocation<'_> {
        let value_size = std::mem::size_of::<T>();
        let bytes = unsafe { std::slice::from_raw_parts(value_ptr.cast::<u8>(), value_size) };
        self.allocate_and_copy_bytes(bytes)
    }

    /// Allocates Emscripten memory and writes `bytes` into it.
    fn allocate_and_copy_bytes(&self, bytes: &[u8]) -> CoreMemoryAllocation<'_> {
        let alloc = self.allocate_memory(bytes.len() as u32);
        self.get_memory_view(alloc.pointer, bytes.len() as u32)
            .set(&Uint8Array::from(bytes), 0);
        alloc
    }

    /// Executes a core call that writes output length via pointer and returns an output pointer.
    fn call_with_len_output<F>(&self, ocg_duel: OCG_Duel, length: *mut u32, call: F) -> *mut c_void
    where
        F: FnOnce(u32) -> u32,
    {
        let length_alloc = self.allocate_memory(4);
        let result_ptr = call(length_alloc.pointer);
        let len = self.read_u32(length_alloc.pointer);

        if !length.is_null() {
            unsafe { *length = len };
        }

        self.copy_output_to_rust_buffer(ocg_duel, result_ptr, len)
    }

    /// Executes a query-like call that needs `OCG_QueryInfo` marshalled to Emscripten memory.
    fn call_query_with_info<F>(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
        call: F,
    ) -> *mut c_void
    where
        F: FnOnce(u32, u32, u32) -> u32,
    {
        let info_alloc = self.copy_pod_to_wasm(info_ptr);
        self.call_with_len_output(ocg_duel, length, |len_ptr| {
            call(ocg_duel as usize as u32, len_ptr, info_alloc.pointer)
        })
    }

    /// Copies a buffer from Emscripten memory into stable Rust-owned storage and returns
    /// a pointer into that storage.
    fn copy_output_to_rust_buffer(&self, ocg_duel: OCG_Duel, ptr: u32, len: u32) -> *mut c_void {
        if ptr == 0 || len == 0 {
            let mut output_buffers = OUTPUT_BUFFER_REGISTRY.lock().unwrap();
            output_buffers.insert(ocg_duel as usize, Vec::new());
            return std::ptr::null_mut();
        }

        let view = self.get_memory_view(ptr, len);
        let mut bytes = vec![0u8; len as usize];
        view.copy_to(&mut bytes);

        let mut output_buffers = OUTPUT_BUFFER_REGISTRY.lock().unwrap();
        output_buffers.insert(ocg_duel as usize, bytes);

        output_buffers
            .get_mut(&(ocg_duel as usize))
            .map(|buf| buf.as_mut_ptr() as *mut c_void)
            .unwrap_or(std::ptr::null_mut())
    }

    /// Removes callback entries associated with payload pointers.
    fn cleanup_callbacks_for_payloads(&self, payload_keys: [usize; 4]) {
        let mut reg = CALLBACK_REGISTRY.lock().unwrap();
        if payload_keys[0] != 0 {
            let _ = reg.card_readers.remove(&payload_keys[0]);
        }
        if payload_keys[1] != 0 {
            let _ = reg.script_readers.remove(&payload_keys[1]);
        }
        if payload_keys[2] != 0 {
            let _ = reg.log_handlers.remove(&payload_keys[2]);
        }
        if payload_keys[3] != 0 {
            let _ = reg.card_done_callbacks.remove(&payload_keys[3]);
        }
    }

    /// Stores callback pointer in registry for payload if payload is non-null.
    fn register_callback_payload<T, F>(&self, payload: *mut c_void, handler: T, select: F)
    where
        T: Copy,
        F: FnOnce(&mut callbacks::CallbackRegistry) -> &mut HashMap<usize, Option<T>>,
    {
        if payload.is_null() {
            return;
        }

        let mut registry = CALLBACK_REGISTRY.lock().unwrap();
        select(&mut registry).insert(payload as usize, Some(handler));
    }

    /// Retrieves callback pointer from registry by payload key.
    fn get_registered_callback<T, F>(payload: u32, select: F) -> Option<T>
    where
        T: Copy,
        F: FnOnce(&callbacks::CallbackRegistry) -> &HashMap<usize, Option<T>>,
    {
        let registry = CALLBACK_REGISTRY.lock().unwrap();
        select(&registry)
            .get(&(payload as usize))
            .copied()
            .flatten()
    }

    /// Reads a null-terminated string from Emscripten memory.
    fn read_wasm_c_string(&self, ptr: u32, max_len: usize) -> Vec<u8> {
        if ptr == 0 {
            return Vec::new();
        }

        let memory = Uint8Array::new(&self.get_wasm_memory().buffer());
        let memory_len = memory.length() as usize;
        let start = ptr as usize;
        if start >= memory_len {
            return Vec::new();
        }

        let mut out = Vec::new();
        let end = (start + max_len).min(memory_len);
        for idx in start..end {
            let byte = memory.get_index(idx as u32);
            if byte == 0 {
                break;
            }
            out.push(byte);
        }

        out
    }

    /// Copies an `OCG_CardData` struct from Rust memory into Emscripten memory.
    fn write_card_data_to_wasm(&self, wasm_ptr: u32, data: &OCG_CardData) {
        let mut wasm_data = *data;

        // `ocgcore` reads this pointer when constructing internal cached card data.
        // We allocate in Emscripten memory so the pointer is valid on the C++ side.
        let setcodes_ptr = if data.setcodes.is_null() {
            0
        } else {
            let mut setcodes: Vec<u16> = Vec::new();
            for i in 0..64 {
                let sc = unsafe { *data.setcodes.add(i) };
                setcodes.push(sc);
                if sc == 0 {
                    break;
                }
            }
            if setcodes.last().copied() != Some(0) {
                setcodes.push(0);
            }

            let bytes_len = (setcodes.len() * std::mem::size_of::<u16>()) as u32;
            let alloc_ptr = self._malloc(bytes_len);
            let setcodes_bytes = unsafe {
                std::slice::from_raw_parts(
                    setcodes.as_ptr().cast::<u8>(),
                    setcodes.len() * std::mem::size_of::<u16>(),
                )
            };

            self.get_memory_view(alloc_ptr, bytes_len)
                .set(&Uint8Array::from(setcodes_bytes), 0);
            alloc_ptr
        };

        wasm_data.setcodes = setcodes_ptr as usize as *mut u16;

        let data_bytes = unsafe {
            std::slice::from_raw_parts(
                (&wasm_data as *const OCG_CardData).cast::<u8>(),
                std::mem::size_of::<OCG_CardData>(),
            )
        };

        self.get_memory_view(wasm_ptr, std::mem::size_of::<OCG_CardData>() as u32)
            .set(&Uint8Array::from(data_bytes), 0);
    }

    /// Prepares the callback functions for injection into Emscripten and returns
    /// an allocation containing the options struct for the duration of the call.
    fn prepare_options_for_wasm(&self, options: &OCG_DuelOptions) -> CoreMemoryAllocation<'_> {
        let options_size = std::mem::size_of::<OCG_DuelOptions>();
        let options_alloc = self.allocate_memory(options_size as u32);

        let mut wasm_options = *options;

        if let Some(handler) = options.cardReader {
            self.register_callback_payload(options.payload1, handler, |reg| &mut reg.card_readers);

            let idx = self.inject_function_into_emscripten("_card_reader_idx", "viii", || {
                let backend = self.clone();
                let cb = Closure::wrap(Box::new(move |payload: u32, code: u32, data: u32| {
                    let handler = Self::get_registered_callback(payload, |reg| &reg.card_readers);

                    if let Some(card_reader) = handler {
                        let mut card_data = unsafe { std::mem::zeroed::<OCG_CardData>() };
                        unsafe {
                            card_reader(payload as *mut c_void, code, &mut card_data as *mut _);
                        }
                        backend.write_card_data_to_wasm(data, &card_data);
                    }
                }) as Box<dyn Fn(u32, u32, u32)>);
                let func = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
                cb.forget();
                func
            });
            wasm_options.cardReader =
                unsafe { std::mem::transmute::<usize, OCG_DataReader>(idx as usize) };
        }

        if let Some(handler) = options.scriptReader {
            self.register_callback_payload(options.payload2, handler, |reg| {
                &mut reg.script_readers
            });

            let idx = self.inject_function_into_emscripten("_script_reader_idx", "iiii", || {
                let backend = self.clone();
                let cb = Closure::wrap(Box::new(move |payload: u32, duel: u32, name: u32| {
                    let handler = Self::get_registered_callback(payload, |reg| &reg.script_readers);

                    let Some(script_reader) = handler else {
                        return 0;
                    };

                    let name_bytes = backend.read_wasm_c_string(name, 4096);
                    let name_cstring = match CString::new(name_bytes) {
                        Ok(value) => value,
                        Err(_) => return 0,
                    };

                    unsafe {
                        script_reader(
                            payload as *mut c_void,
                            duel as usize as OCG_Duel,
                            name_cstring.as_ptr(),
                        )
                    }
                }) as Box<dyn Fn(u32, u32, u32) -> i32>);
                let func = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
                cb.forget();
                func
            });
            wasm_options.scriptReader =
                unsafe { std::mem::transmute::<usize, OCG_ScriptReader>(idx as usize) };
        }

        if let Some(handler) = options.logHandler {
            self.register_callback_payload(options.payload3, handler, |reg| &mut reg.log_handlers);

            let idx = self.inject_function_into_emscripten("_log_idx", "viii", || {
                let backend = self.clone();
                let cb = Closure::wrap(Box::new(move |payload: u32, string: u32, log_type: i32| {
                    let handler = Self::get_registered_callback(payload, |reg| &reg.log_handlers);

                    let Some(log_handler) = handler else {
                        return;
                    };

                    let string_bytes = backend.read_wasm_c_string(string, 16384);
                    let string_cstring = match CString::new(string_bytes) {
                        Ok(value) => value,
                        Err(_) => return,
                    };

                    unsafe {
                        log_handler(payload as *mut c_void, string_cstring.as_ptr(), log_type);
                    }
                }) as Box<dyn Fn(u32, u32, i32)>);
                let func = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
                cb.forget();
                func
            });
            wasm_options.logHandler =
                unsafe { std::mem::transmute::<usize, OCG_LogHandler>(idx as usize) };
        }

        if let Some(handler) = options.cardReaderDone {
            self.register_callback_payload(options.payload4, handler, |reg| {
                &mut reg.card_done_callbacks
            });

            let idx = self.inject_function_into_emscripten("_card_done_idx", "vii", || {
                let backend = self.clone();
                let cb = Closure::wrap(Box::new(move |payload: u32, data: u32| {
                    let handler =
                        Self::get_registered_callback(payload, |reg| &reg.card_done_callbacks);

                    let Some(card_done_handler) = handler else {
                        return;
                    };

                    let mut card_data_bytes = vec![0u8; std::mem::size_of::<OCG_CardData>()];
                    backend
                        .get_memory_view(data, std::mem::size_of::<OCG_CardData>() as u32)
                        .copy_to(&mut card_data_bytes);

                    let mut card_data = unsafe {
                        std::ptr::read_unaligned(card_data_bytes.as_ptr() as *const OCG_CardData)
                    };

                    unsafe {
                        card_done_handler(payload as *mut c_void, &mut card_data as *mut _);
                    }
                }) as Box<dyn Fn(u32, u32)>);
                let func = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
                cb.forget();
                func
            });
            wasm_options.cardReaderDone =
                unsafe { std::mem::transmute::<usize, OCG_DataReaderDone>(idx as usize) };
        }

        let bytes = unsafe {
            std::slice::from_raw_parts(&wasm_options as *const _ as *const u8, options_size)
        };

        let view = self.get_memory_view(options_alloc.pointer, options_size as u32);
        view.set(&Uint8Array::from(bytes), 0);

        options_alloc
    }
}
