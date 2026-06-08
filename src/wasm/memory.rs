use super::WasmBackend;

/// Safe wrapper around a pointer in WebAssembly memory.
///
/// This implements [`Drop`] for you, so it's recommended you use it wherever possible.
///
/// # Lifetime Safety & Pitfalls
/// Because this type implements [`Drop`], the underlying memory is automatically
/// deallocated via Emscripten's `_free` hook as soon as this struct goes out of scope.
///
/// If you extract the raw `pointer` field outside this wrapper, you **must** ensure that this
/// `CoreMemoryAllocation` container remains alive for the entire duration of that usage.
/// Letting this struct drop prematurely will cause a silent use-after-free or memory
/// corruption bug on the WebAssembly heap.
#[derive(Debug, Clone, PartialEq)]
pub struct CoreMemoryAllocation<'a> {
    pub core: &'a WasmBackend,
    pub pointer: u32,
}

impl<'a> Drop for CoreMemoryAllocation<'a> {
    /// Free the allocated memory.
    fn drop(&mut self) {
        self.core.free(self.pointer);
    }
}
