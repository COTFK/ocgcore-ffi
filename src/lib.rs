//! Raw FFI (foreign function interface) library for [edo9300/ygopro-core] (aka `ocgcore`).
//!
//! This crate handles the bare minimum required for Rust applications to interface with
//! `ocgcore`'s C API; users are expected to build their own safe Rust abstractions on top of this crate.
//!
//! # Usage
//! Read the [edo9300/ygopro-core] documentation - this crate mirrors the C API exactly.
//!
//! # WebAssembly
//! For `wasm32-unknown-unknown`, `ocgcore` is compiled using Emscripten to a JavaScript + WebAssembly bundle;
//! that bundle is then interfaced with by [wasm_bindgen]. As such, for this target, additional logic is
//! provided to handle memory management and callback function injection into Emscripten.
//!
//! [edo9300/ygopro-core]: https://github.com/edo9300/ygopro-core

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod native;
pub mod types;
mod wasm;

use std::ffi::c_void;

use types::OCG_Duel;
use types::OCG_DuelOptions;
use types::OCG_NewCardInfo;
use types::OCG_QueryInfo;

/// Wrapper struct that interfaces with the appropriate (native/Wasm) backend.
#[derive(Debug, Clone, PartialEq)]
pub struct OCGCore {
    #[cfg(target_arch = "wasm32")]
    backend: wasm::WasmBackend,
    #[cfg(not(target_arch = "wasm32"))]
    backend: native::NativeBackend,
}

impl OCGCore {
    /// Initializes `ocgcore` for the target platform and returns a handle to it.
    pub async fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                backend: wasm::WasmBackend::new().await,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                backend: native::NativeBackend {},
            }
        }
    }
}

impl OCGCore {
    /// Retrieves the core version and stores it in the provided pointers.
    pub fn OCG_GetVersion(&self, major: &mut i32, minor: &mut i32) {
        self.backend.OCG_GetVersion(major, minor);
    }

    /// Creates a new duel simulation with the specified `options` and saves the pointer in `duel`.
    ///
    /// # Safety
    /// No members of options may be NULL pointers or uninitialized. The `duel` pointer must
    /// point to a valid memory location capable of holding an `OCG_Duel` handle.
    pub unsafe fn OCG_CreateDuel(
        &self,
        duel: *mut OCG_Duel,
        options: *const OCG_DuelOptions,
    ) -> i32 {
        unsafe { self.backend.OCG_CreateDuel(duel, options) }
    }

    /// Adds a new card to the provided duel.
    ///
    /// # Safety
    /// Both `duel` and the `card` pointer must be valid and non-null.
    pub unsafe fn OCG_DuelNewCard(&self, duel: OCG_Duel, card: *const OCG_NewCardInfo) {
        unsafe { self.backend.OCG_DuelNewCard(duel, card) }
    }

    /// Load a Lua script for the specified duel.
    ///
    /// Generally you only need to call this directly for loading global scripts.
    ///
    /// For card scripts, you can call this function in your [`types::OCG_ScriptReader`] handler,
    /// provided to [`OCGCore::OCG_CreateDuel`]. This way, scripts will be loaded automatically
    /// when a card is added to the duel.
    ///
    /// Returns positive on success and zero on failure.
    ///
    /// # Safety
    /// The `script` and `name` pointers must point to valid, null-terminated C-strings.
    pub unsafe fn OCG_LoadScript(
        &self,
        duel: OCG_Duel,
        script: *const std::ffi::c_char,
        length: u32,
        name: *const std::ffi::c_char,
    ) -> i32 {
        unsafe { self.backend.OCG_LoadScript(duel, script, length, name) }
    }

    /// Starts the duel.
    ///
    /// # Safety
    /// `duel` must be a valid, active handle initialized by `OCG_CreateDuel`.
    pub unsafe fn OCG_StartDuel(&self, duel: OCG_Duel) {
        unsafe { self.backend.OCG_StartDuel(duel) }
    }

    /// Processes the next step in the simulation, returning the current status.
    ///
    /// # Safety
    /// `duel` must be a valid, active handle.
    pub unsafe fn OCG_DuelProcess(&self, duel: OCG_Duel) -> i32 {
        unsafe { self.backend.OCG_DuelProcess(duel) }
    }

    /// Returns a pointer to the message buffer, and writes the current message length in the provided pointer.
    ///
    /// Use this to read messages from the core; subsequent calls invalidate previous buffers, so making copies
    /// is recommended.
    ///
    /// # Safety
    /// The `length` pointer must be valid and mutable. The returned pointer points to internal core memory
    /// and must not be mutated or freed manually.
    pub unsafe fn OCG_DuelGetMessage(&self, duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        unsafe { self.backend.OCG_DuelGetMessage(duel, length) }
    }

    /// Sets the next player response for the duel simulation.
    /// Subsequent calls overwrite previous responses if not processed.
    /// The contents of the provided `response` buffer are copied internally, assuming it contains `length` bytes.
    ///
    /// # Safety
    /// The `response` pointer must point to a valid block of memory at least `length` bytes wide.
    pub unsafe fn OCG_DuelSetResponse(&self, duel: OCG_Duel, response: *const c_void, length: u32) {
        unsafe { self.backend.OCG_DuelSetResponse(duel, response, length) }
    }

    /// Returns the number of cards in the specified zone.
    ///
    /// # Safety
    /// `duel` must point to an active, initialized duel instance.
    pub unsafe fn OCG_DuelQueryCount(&self, duel: OCG_Duel, team: u8, location: u32) -> u32 {
        unsafe { self.backend.OCG_DuelQueryCount(duel, team, location) }
    }

    /// Returns a pointer to a buffer with the FIRST card found in the duel that matches the query.
    /// The size of the buffer is written to `length` if the pointer is not null.
    /// Subsequent calls invalidate previous queries.
    ///
    /// # Safety
    /// `query` must be a valid pointer to an initialized `OCG_QueryInfo` instance.
    pub unsafe fn OCG_DuelQuery(
        &self,
        duel: OCG_Duel,
        length: *mut u32,
        query: *const OCG_QueryInfo,
    ) -> *mut c_void {
        unsafe { self.backend.OCG_DuelQuery(duel, length, query) }
    }

    /// Returns a pointer to a buffer containing card counts for every zone in the game.
    /// The size of the buffer is written to `length` if the pointer is not null.
    /// Subsequent calls invalidate previous queries.
    ///
    /// # Safety
    /// `duel` must be a valid, live handle.
    pub unsafe fn OCG_DuelQueryField(&self, duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        unsafe { self.backend.OCG_DuelQueryField(duel, length) }
    }

    /// Returns a pointer to a buffer with ALL the cards matching the query.
    /// The size of the buffer is written to `length` if the pointer is not null.
    /// Subsequent calls invalidate previous queries.
    ///
    /// # Safety
    /// `query` must point to a valid `OCG_QueryInfo` layout.
    pub unsafe fn OCG_DuelQueryLocation(
        &self,
        duel: OCG_Duel,
        length: *mut u32,
        query: *const OCG_QueryInfo,
    ) -> *mut c_void {
        unsafe { self.backend.OCG_DuelQueryLocation(duel, length, query) }
    }

    /// Deallocates the duel instance created by [`OCGCore::OCG_CreateDuel()`].
    ///
    /// # Safety
    /// `duel` must be a valid pointer that was explicitly generated by `OCG_CreateDuel` and
    /// has not been destroyed yet.
    pub unsafe fn OCG_DestroyDuel(&self, duel: OCG_Duel) {
        unsafe { self.backend.OCG_DestroyDuel(duel) }
    }
}
