//! Raw FFI (foreign function interface) library for [`edo9300/ygopro-core`] (aka `ocgcore`).
//!
//! This crate handles the bare minimum required for Rust applications to interface with
//! `ocgcore`'s C API; users are expected to build their own safe Rust abstractions on top of this crate.
//!
//! # Usage
//! Read the [`edo9300/ygopro-core`] documentation - this crate mirrors the C API exactly.
//!
//! # WebAssembly
//! For `wasm32-unknown-unknown`, `ocgcore` is compiled using Emscripten to a JavaScript + WebAssembly bundle;
//! that bundle is then interfaced with by [`wasm_bindgen`]. As such, for this target, additional logic is
//! provided to handle memory management and callback function injection into Emscripten.
//! 
//! To initialize the WebAssembly module, call the async [`initialize()`] function. This will load the module and prepare it for use.
//!
//! [`edo9300/ygopro-core`]: https://github.com/edo9300/ygopro-core

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod api;
pub mod types;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

pub use api::*;

/// Initializes global backend state.
///
/// On native targets this is a no-op; on wasm this loads the module.
pub async fn initialize() {
    #[cfg(target_arch = "wasm32")]
    {
        let backend = wasm::WasmBackend::new().await;
        wasm::WASM_BACKEND.with(|backend_cell| {
            *backend_cell.borrow_mut() = Some(backend);
        });
    }
}
