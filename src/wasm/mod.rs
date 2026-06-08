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
use std::ffi::c_char;
use std::ffi::c_void;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

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
use callbacks::wasm_card_done_trampoline;
use callbacks::wasm_card_reader_trampoline;
use callbacks::wasm_log_trampoline;
use callbacks::wasm_script_reader_trampoline;

/// Raw wasm-bindgen FFI - these methods need additional memory management,
/// which the impl block on WasmBackend handles.
#[wasm_bindgen(module = "/ocgcore.js")]
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

        let wasm_options_ptr = self.prepare_options_for_wasm(unsafe { &*options_ptr });
        let handle_storage_ptr = self.allocate_memory(4);
        let status = self._OCG_CreateDuel(handle_storage_ptr.pointer, wasm_options_ptr);

        // If successful, extract the handle and port over callbacks;
        // if not, erase callbacks and let the status code propagate
        if status == 0 {
            let view = self.get_memory_view(handle_storage_ptr.pointer, 1);
            unsafe {
            *out_ocg_duel = view.get_index(0) as OCG_Duel;
            }

            let mut reg = CALLBACK_REGISTRY.lock().unwrap();
            if let Some(cbs) = reg.card_done_callbacks.remove(&(wasm_options_ptr as usize)) {
                reg.card_done_callbacks.insert(out_ocg_duel as usize, cbs);
            }
            if let Some(cbs) = reg.card_readers.remove(&(wasm_options_ptr as usize)) {
                reg.card_readers.insert(out_ocg_duel as usize, cbs);
            }
            if let Some(cbs) = reg.script_readers.remove(&(wasm_options_ptr as usize)) {
                reg.script_readers.insert(out_ocg_duel as usize, cbs);
            }
            if let Some(cbs) = reg.log_handlers.remove(&(wasm_options_ptr as usize)) {
                reg.log_handlers.insert(out_ocg_duel as usize, cbs);
            }
        } else {
            let mut reg = CALLBACK_REGISTRY.lock().unwrap();
            let _ = reg.card_readers.remove(&(wasm_options_ptr as usize));
            let _ = reg.card_done_callbacks.remove(&(wasm_options_ptr as usize));
            let _ = reg.script_readers.remove(&(wasm_options_ptr as usize));
            let _ = reg.log_handlers.remove(&(wasm_options_ptr as usize));
        }

        status
    }

    pub unsafe fn OCG_DestroyDuel(&self, ocg_duel: OCG_Duel) {
        self._OCG_DestroyDuel(ocg_duel as usize as u32);

        // Clean up callbacks
        let mut reg = CALLBACK_REGISTRY.lock().unwrap();
        let _ = reg.card_readers.remove(&(ocg_duel as usize));
        let _ = reg.card_done_callbacks.remove(&(ocg_duel as usize));
        let _ = reg.script_readers.remove(&(ocg_duel as usize));
        let _ = reg.log_handlers.remove(&(ocg_duel as usize));
    }

    pub unsafe fn OCG_DuelNewCard(&self, ocg_duel: OCG_Duel, info_ptr: *const OCG_NewCardInfo) {
        let info_size = std::mem::size_of::<OCG_NewCardInfo>();
        let info_bytes = unsafe { std::slice::from_raw_parts(info_ptr.cast::<u8>(), info_size) };

        let info_alloc = self.allocate_memory(info_size as u32);

        let view = self.get_memory_view(info_alloc.pointer, info_size as u32);
        view.set(&Uint8Array::from(info_bytes), 0);

        self._OCG_DuelNewCard(ocg_duel as usize as u32, info_alloc.pointer);
    }

    pub unsafe fn OCG_StartDuel(&self, ocg_duel: OCG_Duel) {
        self._OCG_StartDuel(ocg_duel as usize as u32);
    }

    pub unsafe fn OCG_DuelProcess(&self, ocg_duel: OCG_Duel) -> i32 {
        self._OCG_DuelProcess(ocg_duel as usize as u32) as i32
    }

    pub unsafe fn OCG_DuelGetMessage(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        let length_alloc = self.allocate_memory(4);
        let message =
            self._OCG_DuelGetMessage(ocg_duel as usize as u32, length_alloc.pointer) as *mut c_void;
        let len = self.get_memory_view(length_alloc.pointer, 0).get_index(0) as u32;

        if !length.is_null() {
            unsafe { *length = len };
        }

        message
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

        let response_alloc = self.allocate_memory(length);

        let response_bytes =
            unsafe { std::slice::from_raw_parts(buffer.cast::<u8>(), length as usize) };

        let view = self.get_memory_view(response_alloc.pointer, length);
        view.set(&Uint8Array::from(response_bytes), 0);

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
            let alloc = self.allocate_memory(length);
            let bytes = unsafe { std::slice::from_raw_parts(buffer as *const u8, length as usize) };

            let view = self.get_memory_view(alloc.pointer, length);
            view.set(&Uint8Array::from(bytes), 0);
            Some(alloc)
        };
        let buffer_ptr = buffer_alloc.as_ref().map(|a| a.pointer).unwrap_or(0);

        let name_alloc = if name.is_null() {
            None
        } else {
            let mut len = 0;

            while len < 4096 && unsafe { *name.add(len) != 0 } {
                len += 1;
            }

            let alloc = self.allocate_memory(len as u32 + 1);
            let bytes = unsafe { std::slice::from_raw_parts(name as *const u8, len + 1) };

            let view = self.get_memory_view(alloc.pointer, len as u32);
            view.set(&Uint8Array::from(bytes), 0);
            Some(alloc)
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
        let info_size = std::mem::size_of::<OCG_QueryInfo>();
        let info_alloc = self.allocate_memory(info_size as u32);
        let info_bytes = unsafe { std::slice::from_raw_parts(info_ptr.cast::<u8>(), info_size) };

        let view = self.get_memory_view(info_alloc.pointer, info_size as u32);
        view.set(&Uint8Array::from(info_bytes), 0);

        let length_alloc = self.allocate_memory(4);
        let result_ptr = self._OCG_DuelQuery(
            ocg_duel as usize as u32,
            length_alloc.pointer,
            info_alloc.pointer,
        );
        let len = self.get_memory_view(length_alloc.pointer, 0).get_index(0) as u32;

        if !length.is_null() {
            unsafe { *length = len };
        }

        result_ptr as *mut c_void
    }

    pub unsafe fn OCG_DuelQueryLocation(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void {
        let info_size = std::mem::size_of::<OCG_QueryInfo>();
        let info_alloc = self.allocate_memory(info_size as u32);
        let info_bytes = unsafe { std::slice::from_raw_parts(info_ptr.cast::<u8>(), info_size) };

        let view = self.get_memory_view(info_alloc.pointer, info_size as u32);
        view.set(&Uint8Array::from(info_bytes), 0);

        let length_alloc = self.allocate_memory(4);
        let result_ptr = self._OCG_DuelQueryLocation(
            ocg_duel as usize as u32,
            length_alloc.pointer,
            info_alloc.pointer,
        );
        let len = self.get_memory_view(length_alloc.pointer, 0).get_index(0) as u32;

        if !length.is_null() {
            unsafe { *length = len };
        }

        result_ptr as *mut c_void
    }

    pub unsafe fn OCG_DuelQueryField(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        let length_alloc = self.allocate_memory(4);
        let result_ptr = self._OCG_DuelQueryField(ocg_duel as usize as u32, length_alloc.pointer);
        let len = self.get_memory_view(length_alloc.pointer, 0).get_index(0) as u32;

        if !length.is_null() {
            unsafe { *length = len };
        }

        result_ptr as *mut c_void
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

    /// Prepares the callback functions for injection into Emscripten, and returns the pointer 
    /// to the allocated memory in WebAssembly.
    fn prepare_options_for_wasm(&self, options: &OCG_DuelOptions) -> u32 {
        let options_size = std::mem::size_of::<OCG_DuelOptions>();
        let options_alloc = self.allocate_memory(options_size as u32);

        let mut wasm_options = *options;

        if let Some(handler) = options.cardReader {
            let mut registry = CALLBACK_REGISTRY.lock().unwrap();
            registry
                .card_readers
                .insert(options_alloc.pointer as usize, Some(handler));

            let idx = self.inject_function_into_emscripten("_card_reader_idx", "viii", || {
                let cb = Closure::wrap(Box::new(|payload: u32, code: u32, data: u32| unsafe {
                    wasm_card_reader_trampoline(payload as *mut c_void, code, data as *mut _)
                }) as Box<dyn Fn(u32, u32, u32)>);
                let func = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
                cb.forget();
                func
            });
            wasm_options.cardReader =
                unsafe { std::mem::transmute::<usize, OCG_DataReader>(idx as usize) };
        }

        if let Some(handler) = options.scriptReader {
            let mut registry = CALLBACK_REGISTRY.lock().unwrap();
            registry
                .script_readers
                .insert(options_alloc.pointer as usize, Some(handler));

            let idx = self.inject_function_into_emscripten("_script_reader_idx", "iiii", || {
                let cb = Closure::wrap(Box::new(|payload: u32, duel: u32, name: u32| unsafe {
                    wasm_script_reader_trampoline(
                        payload as *mut c_void,
                        duel as OCG_Duel,
                        name as *const c_char,
                    )
                }) as Box<dyn Fn(u32, u32, u32) -> i32>);
                let func = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
                cb.forget();
                func
            });
            wasm_options.scriptReader =
                unsafe { std::mem::transmute::<usize, OCG_ScriptReader>(idx as usize) };
        }

        if let Some(handler) = options.logHandler {
            let mut registry = CALLBACK_REGISTRY.lock().unwrap();
            registry
                .log_handlers
                .insert(options_alloc.pointer as usize, Some(handler));

            let idx = self.inject_function_into_emscripten("_log_idx", "viii", || {
                let cb =
                    Closure::wrap(Box::new(|payload: u32, string: u32, log_type: i32| unsafe {
                        wasm_log_trampoline(
                            payload as *mut c_void,
                            string as *const c_char,
                            log_type,
                        )
                    }) as Box<dyn Fn(u32, u32, i32)>);
                let func = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
                cb.forget();
                func
            });
            wasm_options.logHandler =
                unsafe { std::mem::transmute::<usize, OCG_LogHandler>(idx as usize) };
        }

        if let Some(handler) = options.cardReaderDone {
            let mut registry = CALLBACK_REGISTRY.lock().unwrap();
            registry
                .card_done_callbacks
                .insert(options_alloc.pointer as usize, Some(handler));

            let idx = self.inject_function_into_emscripten("_card_done_idx", "vii", || {
                let cb = Closure::wrap(Box::new(|payload: u32, data: u32| unsafe {
                    wasm_card_done_trampoline(payload as *mut c_void, data as *mut _)
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

        options_alloc.pointer
    }
}
