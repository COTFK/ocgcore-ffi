use super::WasmCore;

#[derive(Debug, Clone, PartialEq)]
pub struct CoreMemoryAllocation<'a> {
    pub core: &'a WasmCore,
    pub pointer: u32,
}

impl<'a> Drop for CoreMemoryAllocation<'a> {
    /// Free the allocated memory.
    fn drop(&mut self) {
        self.core.free(self.pointer);
    }
}
