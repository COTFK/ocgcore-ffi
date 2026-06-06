mod memory;

use js_sys::Uint8Array;
use js_sys::Uint32Array;
use js_sys::futures::JsFuture;
use std::ffi::c_void;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;

use crate::backend::OCGCoreBackend;
use crate::types::OCG_Duel;
use crate::types::OCG_DuelOptions;
use crate::types::OCG_NewCardInfo;
use crate::types::OCG_QueryInfo;
use memory::CoreMemoryAllocation;

#[wasm_bindgen(module = "/ocgcore.js")]
extern "C" {
    #[derive(Debug, Clone, PartialEq)]
    pub type WasmCore;

    #[wasm_bindgen(js_name = default)]
    fn _init() -> js_sys::Promise;

    #[wasm_bindgen(method)]
    fn _OCG_GetVersion(this: &WasmCore, major: u32, minor: u32);

    #[wasm_bindgen(method)]
    fn _OCG_CreateDuel(this: &WasmCore, duel: u32, options: u32) -> i32;

    #[wasm_bindgen(method)]
    fn _OCG_StartDuel(this: &WasmCore, duel: u32);

    #[wasm_bindgen(method)]
    fn _OCG_DuelProcess(this: &WasmCore, duel: u32) -> u32;

    #[wasm_bindgen(method)]
    fn _OCG_DuelGetMessage(this: &WasmCore, duel: u32, length_ptr: u32) -> u32;

    #[wasm_bindgen(method)]
    fn _OCG_DuelSetResponse(this: &WasmCore, duel: u32, response_ptr: u32, len: u32);

    #[wasm_bindgen(method)]
    fn _OCG_DuelNewCard(this: &WasmCore, duel: u32, info: u32);

    #[wasm_bindgen(method)]
    fn _OCG_LoadScript(this: &WasmCore, duel: u32, buffer: u32, len: u32, name: u32) -> i32;

    #[wasm_bindgen(method)]
    fn _OCG_DestroyDuel(this: &WasmCore, duel: u32);

    #[wasm_bindgen(method)]
    fn _OCG_DuelQueryCount(this: &WasmCore, duel: u32, team: u8, location: u32) -> u32;

    #[wasm_bindgen(method)]
    fn _OCG_DuelQueryLocation(this: &WasmCore, duel: u32, length_ptr: u32, info_ptr: u32) -> u32;

    #[wasm_bindgen(method)]
    fn _OCG_DuelQueryField(this: &WasmCore, duel: u32, length_ptr: u32) -> u32;

    // Emscripten helpers
    #[wasm_bindgen(method, getter, js_name = wasmMemory)]
    fn get_wasm_memory(this: &WasmCore) -> js_sys::WebAssembly::Memory;

    #[wasm_bindgen(method, js_name = _malloc)]
    fn malloc(this: &WasmCore, size: u32) -> u32;

    #[wasm_bindgen(method, js_name = _free)]
    fn free(this: &WasmCore, ptr: u32);

    #[wasm_bindgen(method, js_name = addFunction)]
    fn add_function(this: &WasmCore, func: &js_sys::Function, signature: &str) -> u32;
}

impl WasmCore {
    pub async fn new() -> Self {
        let promise = _init();

        let ocgcore = JsFuture::from(promise).await.unwrap();

        ocgcore.unchecked_into()
    }
}

impl OCGCoreBackend for WasmCore {
    fn OCG_GetVersion(&self, major: &mut i32, minor: &mut i32) {
        let major_alloc = CoreMemoryAllocation::new(self, std::mem::size_of::<i32>());
        let minor_alloc = CoreMemoryAllocation::new(self, std::mem::size_of::<i32>());

        let major_ptr = major_alloc.get_pointer();
        let minor_ptr = minor_alloc.get_pointer();

        self._OCG_GetVersion(major_ptr.into(), minor_ptr.into());

        let memory = self.get_wasm_memory();
        let buffer: js_sys::ArrayBuffer = memory.buffer().into();
        let view = js_sys::DataView::new(&buffer, 0, buffer.byte_length() as usize);

        *major = view.get_int32_endian(major_ptr.into(), true);
        *minor = view.get_int32_endian(minor_ptr.into(), true);
    }

    fn OCG_CreateDuel(
        &self,
        out_ocg_duel: *mut OCG_Duel,
        options_ptr: *const OCG_DuelOptions,
    ) -> i32 {
        let options_size = std::mem::size_of::<OCG_DuelOptions>();
        let options_alloc = CoreMemoryAllocation::new(self, options_size);
        let wasm_options_ptr = options_alloc.get_pointer();

        let options_bytes =
            unsafe { std::slice::from_raw_parts(options_ptr.cast::<u8>(), options_size) };
        let memory = self.get_wasm_memory();
        let dest_view = Uint8Array::new_with_byte_offset_and_length(
            &memory.buffer(),
            wasm_options_ptr.into(),
            options_size as u32,
        );
        dest_view.set(&Uint8Array::from(options_bytes), 0);

        let handle_storage_ptr = self.malloc(4);
        let status = self._OCG_CreateDuel(handle_storage_ptr, wasm_options_ptr.into());
        let view =
            Uint32Array::new_with_byte_offset_and_length(&memory.buffer(), handle_storage_ptr, 1);
        let duel_handle = view.get_index(0);

        if !out_ocg_duel.is_null() {
            unsafe {
                *out_ocg_duel = duel_handle as OCG_Duel;
            }
        }

        self.free(handle_storage_ptr);

        status
    }

    fn OCG_DestroyDuel(&self, ocg_duel: OCG_Duel) {
        self._OCG_DestroyDuel(ocg_duel as usize as u32);
    }

    fn OCG_DuelNewCard(&self, ocg_duel: OCG_Duel, info_ptr: *const OCG_NewCardInfo) {
        let info_size = std::mem::size_of::<OCG_NewCardInfo>();
        let info_bytes = unsafe { std::slice::from_raw_parts(info_ptr.cast::<u8>(), info_size) };

        let info_alloc = CoreMemoryAllocation::new(self, info_size);
        let info_offset = info_alloc.get_pointer().into();

        let memory = self.get_wasm_memory();
        let memory_buf = memory.buffer();

        let info_view =
            Uint8Array::new_with_byte_offset_and_length(&memory_buf, info_offset, info_size as u32);
        info_view.set(&Uint8Array::from(info_bytes), 0);

        self._OCG_DuelNewCard(ocg_duel as usize as u32, info_offset);
    }

    fn OCG_StartDuel(&self, ocg_duel: OCG_Duel) {
        self._OCG_StartDuel(ocg_duel as usize as u32);
    }

    fn OCG_DuelProcess(&self, ocg_duel: OCG_Duel) -> i32 {
        self._OCG_DuelProcess(ocg_duel as usize as u32) as i32
    }

    fn OCG_DuelGetMessage(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        let length_alloc = CoreMemoryAllocation::new(self, 4);

        let message = self
            ._OCG_DuelGetMessage(ocg_duel as usize as u32, length_alloc.get_pointer().into())
            as *mut c_void;

        let len = length_alloc.read_u32(0, &self.get_wasm_memory());

        if !length.is_null() {
            unsafe { *length = len };
        }

        message
    }

    fn OCG_DuelSetResponse(&self, ocg_duel: OCG_Duel, buffer: *const c_void, length: u32) {
        if buffer.is_null() || length == 0 {
            self._OCG_DuelSetResponse(ocg_duel as usize as u32, 0, 0);
            return;
        }

        let response_alloc = CoreMemoryAllocation::new(self, length as usize);
        let response_ptr = response_alloc.get_pointer();

        let response_bytes =
            unsafe { std::slice::from_raw_parts(buffer.cast::<u8>(), length as usize) };

        let memory = self.get_wasm_memory();
        let view = Uint8Array::new_with_byte_offset_and_length(
            &memory.buffer(),
            response_ptr.into(),
            length,
        );
        view.set(&Uint8Array::from(response_bytes), 0);

        self._OCG_DuelSetResponse(ocg_duel as usize as u32, response_ptr.into(), length);
    }

    fn OCG_LoadScript(
        &self,
        ocg_duel: OCG_Duel,
        buffer: *const std::ffi::c_char,
        length: u32,
        name: *const std::ffi::c_char,
    ) -> i32 {
        let memory = self.get_wasm_memory();
        let memory_buf = memory.buffer();

        let buffer_alloc = CoreMemoryAllocation::new(self, length as usize);
        let buffer_ptr = buffer_alloc.get_pointer();
        let buffer_bytes =
            unsafe { std::slice::from_raw_parts(buffer as *const u8, length as usize) };

        let buffer_view =
            Uint8Array::new_with_byte_offset_and_length(&memory_buf, buffer_ptr.into(), length);
        buffer_view.set(&Uint8Array::from(buffer_bytes), 0);

        let name_len = unsafe { strlen(name) };
        let name_alloc = CoreMemoryAllocation::new(self, name_len + 1);
        let name_ptr = name_alloc.get_pointer();

        let name_bytes = unsafe { std::slice::from_raw_parts(name as *const u8, name_len + 1) };
        let name_view = Uint8Array::new_with_byte_offset_and_length(
            &memory_buf,
            name_ptr.into(),
            (name_len + 1) as u32,
        );
        name_view.set(&Uint8Array::from(name_bytes), 0);

        let result = self._OCG_LoadScript(
            ocg_duel as usize as u32,
            buffer_ptr.into(),
            length,
            name_ptr.into(),
        );

        result
    }
    fn OCG_DuelQueryCount(&self, ocg_duel: OCG_Duel, team: u8, loc: u32) -> u32 {
        self._OCG_DuelQueryCount(ocg_duel as usize as u32, team, loc)
    }
    fn OCG_DuelQuery(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void {
        let memory = self.get_wasm_memory();
        let memory_buf = memory.buffer();

        let info_size = std::mem::size_of::<OCG_QueryInfo>();
        let info_alloc = CoreMemoryAllocation::new(self, info_size);
        let wasm_info_ptr = info_alloc.get_pointer();

        let info_bytes = unsafe { std::slice::from_raw_parts(info_ptr.cast::<u8>(), info_size) };
        let info_view = Uint8Array::new_with_byte_offset_and_length(
            &memory_buf,
            wasm_info_ptr.into(),
            info_size as u32,
        );
        info_view.set(&Uint8Array::from(info_bytes), 0);

        let length_alloc = CoreMemoryAllocation::new(self, 4);

        let result_ptr = self._OCG_DuelQueryLocation(
            ocg_duel as usize as u32,
            length_alloc.get_pointer().into(),
            wasm_info_ptr.into(),
        );

        let len = length_alloc.read_u32(0, &memory);
        if !length.is_null() {
            unsafe { *length = len };
        }

        result_ptr as *mut c_void
    }
    fn OCG_DuelQueryLocation(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void {
        let memory = self.get_wasm_memory();
        let memory_buf = memory.buffer();

        let info_size = std::mem::size_of::<OCG_QueryInfo>();
        let info_alloc = CoreMemoryAllocation::new(self, info_size);
        let wasm_info_ptr = info_alloc.get_pointer();

        let info_bytes = unsafe { std::slice::from_raw_parts(info_ptr.cast::<u8>(), info_size) };
        let info_view = Uint8Array::new_with_byte_offset_and_length(
            &memory_buf,
            wasm_info_ptr.into(),
            info_size as u32,
        );
        info_view.set(&Uint8Array::from(info_bytes), 0);

        let length_alloc = CoreMemoryAllocation::new(self, 4);

        let result_ptr = self._OCG_DuelQueryLocation(
            ocg_duel as usize as u32,
            length_alloc.get_pointer().into(),
            wasm_info_ptr.into(),
        );

        let len = length_alloc.read_u32(0, &memory);
        if !length.is_null() {
            unsafe { *length = len };
        }

        result_ptr as *mut c_void
    }
    fn OCG_DuelQueryField(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        let memory = self.get_wasm_memory();

        let length_alloc = CoreMemoryAllocation::new(self, 4);

        let result_ptr =
            self._OCG_DuelQueryField(ocg_duel as usize as u32, length_alloc.get_pointer().into());

        let len = length_alloc.read_u32(0, &memory);
        if !length.is_null() {
            unsafe { *length = len };
        }

        result_ptr as *mut c_void
    }
}

unsafe fn strlen(ptr: *const std::ffi::c_char) -> usize {
    let mut len = 0;

    unsafe {
        while *ptr.add(len) != 0 {
            len += 1;
        }
    }

    len
}
