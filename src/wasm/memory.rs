/// WASM memory management tools.
use std::ops::Add;
use wasm_bindgen::JsValue;
use js_sys::WebAssembly::Memory;

use crate::wasm::WasmCore;

/// A safe wrapper around an address in WASM memory.
/// Preferable to accidentally feeding the FFI
/// a random [`u32`] that doesn't belong.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CorePointer(pub(super) u32);

impl CorePointer {
    pub const fn new(address: u32) -> Self {
        Self(address)
    }

    pub fn offset_by(&self, offset: usize) -> Self {
        *self + offset
    }
}

impl From<CorePointer> for usize {
    fn from(ptr: CorePointer) -> Self {
        ptr.0 as Self
    }
}

impl From<CorePointer> for u32 {
    fn from(ptr: CorePointer) -> Self {
        ptr.0
    }
}

impl From<CorePointer> for JsValue {
    fn from(ptr: CorePointer) -> Self {
        Self::from_f64(f64::from(ptr.0))
    }
}

impl Add<usize> for CorePointer {
    type Output = Self;
    fn add(self, rhs: usize) -> Self {
        Self(self.0 + rhs as u32)
    }
}

/// Manually allocated memory in WASM.
///
/// [`WASMMemoryAllocation::new()`] sets up the _malloc call,
/// and the [`Drop`] implementation handles the _free call automatically.
///
#[derive(Debug)]
pub struct CoreMemoryAllocation<'a> {
    core: &'a WasmCore,
    pointer: CorePointer,
}

impl<'a> CoreMemoryAllocation<'a> {
    pub fn new(core: &'a WasmCore, length: usize) -> Self {
        Self {
            core,
            pointer: CorePointer::new(core.malloc(length as u32)),
        }
    }

    pub const fn get_pointer(&self) -> CorePointer {
        self.pointer
    }

    pub fn read_u32(&self, offset: usize, memory: &Memory) -> u32 {
        let buffer = memory.buffer();
        let view = js_sys::Uint32Array::new_with_byte_offset_and_length(
            &buffer, 
            (self.pointer.0 as usize + offset) as u32, 
            1
        );
        view.get_index(0)
    }
}

impl Drop for CoreMemoryAllocation<'_> {
    /// Free the allocated memory.
    fn drop(&mut self) {
        self.core.free(self.pointer.0);
    }
}
