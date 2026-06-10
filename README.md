# `ocgcore-ffi`

This crate exposes low-level Rust bindings to the C API of [`edo9300/ygopro-core`] (aka `ocgcore`).

Only the bare minimum[^1] required for Rust applications to interface with the C API is provided; users are expected to implement their own safe abstractions on top of this crate.

## Usage
First and foremost, read the [`edo9300/ygopro-core`] documentation - this crate mirrors the C API exactly.

### Add the crate to your project
In your `Cargo.toml`:
```toml
[dependencies]
# The `bundled` feature enables automatic compilation and linking of `ocgcore`.
# If you do not activate it, you'll have to provide ocgcore yourself.
# This can be helpful for certain targets (i.e. WebAssembly).
ocgcore-ffi = { version = "1.0.0", features = ["bundled"] }
```

>[!NOTE]
> For WebAssembly targets (`wasm32-unknown-unknown`), read the [Notes on `ocgcore` vendoring](#notes-on-ocgcore-vendoring) section.

### Code example

```rust
use std::ffi::c_void;
use std::ptr::null_mut;

use ocgcore_ffi::types::OCG_DuelOptions;
use ocgcore_ffi::types::OCG_NewCardInfo;
use ocgcore_ffi::*;

fn main() {
    // For WebAssembly targets, call `ocgcore_ffi::initialize()` in an async context here.
    // **Forgetting to do this will cause the functions to panic.**

    // Create duel
    let mut duel = null_mut();
    let options = OCG_DuelOptions { /* options */ };
    let creation_status = unsafe { OCG_CreateDuel(&mut duel, &options) };

    // Add a card to the duel
    let card = OCG_NewCardInfo { /* card data */ };
    unsafe { OCG_DuelNewCard(duel, &card) };

    // Start duel
    unsafe { OCG_StartDuel(duel) };

    // Process the simulation
    let duel_status = unsafe { OCG_DuelProcess(duel) };

    // Retrieve messages from the core
    let mut message_length = 0;
    let messages = unsafe { OCG_DuelGetMessage(duel, &mut message_length) };

    // Send a response
    let response = [1, 0, 0, 0];
    let length = 4;
    unsafe { OCG_DuelSetResponse(duel, response.as_ptr() as *const c_void, length) };

    // End the duel
    unsafe { OCG_DestroyDuel(duel) };
}
```

## Notes on `ocgcore` vendoring

The `bundled` feature enables automatic compilation and linking of `ocgcore` during build. **This is disabled by default.**

For native builds, however, we recommend enabling it as it simplifies the work you need to do to use this crate.
If you keep it disabled, you need to make sure you compile and link `ocgcore` yourself during build.

### WebAssembly
For the `bundled` feature to work, you will need to install [Emscripten] and make sure the `EMSDK` environment variable is set. We will use it to cross-compile `ocgcore` and load the output as a JS snippet using `wasm-bindgen`.

This may be less performant than building and bundling it yourself, due to using the `-sWASM=0` parameter to generate a standalone JS file (see [`src/build.rs`](src/build.rs)).

In your own project, you should use `-sWASM=1` to generate a `*.wasm` binary, plus the `*.js` helper required by Emscripten, both of which should be smaller and more performant than the standalone JS file.

However, you need to configure your web framework or bundler to place these files at `/ocgcore.wasm` and `/ocgcore.js` respectively, in your final bundle. If that isn't possible or wanted, the `bundled` feature will still work just fine, albeit a little slower.

## Contributing
We accept contributions! Reach out to us in the [Circle of the Fire Kings Discord server](https://discord.gg/8JtxHUAdGq) to chat and discuss your proposed changes!

## License
`ocgcore-ffi` is licensed under [the AGPL-3.0 license](LICENSE).

Unless explicitly stated otherwise, any contribution to this project shall be licensed under the aforementioned license, without any additional terms or conditions.

[^1]: For native targets. For WebAssembly (`wasm32-unknown-unknown`), a light wrapper is added that handles memory and Rust <-> WebAssembly glue.

[Emscripten]: https://emscripten.org/
[`edo9300/ygopro-core`]: https://github.com/edo9300/ygopro-core