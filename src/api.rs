use std::ffi::c_char;
use std::ffi::c_void;

use super::types::OCG_Duel;
use super::types::OCG_DuelOptions;
use super::types::OCG_NewCardInfo;
use super::types::OCG_QueryInfo;

/// Retrieves the core version and stores it in the provided pointers.
pub fn OCG_GetVersion(major: &mut i32, minor: &mut i32) {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| backend.OCG_GetVersion(major, minor));
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_GetVersion(major, minor);
    }
}

/// Creates a new duel simulation with the specified `options` and saves the pointer in `duel`.
///
/// # Safety
/// No members of options may be NULL pointers or uninitialized. The `duel` pointer must
/// point to a valid memory location capable of holding an `OCG_Duel` handle.
pub unsafe fn OCG_CreateDuel(duel: *mut OCG_Duel, options: *const OCG_DuelOptions) -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| unsafe { backend.OCG_CreateDuel(duel, options) })
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_CreateDuel(duel, options)
    }
}

/// Adds a new card to the provided duel.
///
/// # Safety
/// Both `duel` and the `card` pointer must be valid and non-null.
pub unsafe fn OCG_DuelNewCard(duel: OCG_Duel, card: *const OCG_NewCardInfo) {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| unsafe { backend.OCG_DuelNewCard(duel, card) });
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_DuelNewCard(duel, card);
    }
}

/// Load a Lua script for the specified duel.
///
/// Generally you only need to call this directly for loading global scripts.
///
/// For card scripts, you can call this function in your [`types::OCG_ScriptReader`] handler,
/// provided to [`OCG_CreateDuel`]. This way, scripts will be loaded automatically
/// when a card is added to the duel.
///
/// Returns positive on success and zero on failure.
///
/// # Safety
/// The `script` and `name` pointers must point to valid, null-terminated C-strings.
pub unsafe fn OCG_LoadScript(
    duel: OCG_Duel,
    script: *const c_char,
    length: u32,
    name: *const c_char,
) -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| unsafe {
            backend.OCG_LoadScript(duel, script, length, name)
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_LoadScript(duel, script, length, name)
    }
}

/// Starts the duel.
///
/// # Safety
/// `duel` must be a valid, active handle initialized by `OCG_CreateDuel`.
pub unsafe fn OCG_StartDuel(duel: OCG_Duel) {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| unsafe { backend.OCG_StartDuel(duel) });
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_StartDuel(duel);
    }
}

/// Processes the next step in the simulation, returning the current status.
///
/// # Safety
/// `duel` must be a valid, active handle.
pub unsafe fn OCG_DuelProcess(duel: OCG_Duel) -> i32 {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| unsafe { backend.OCG_DuelProcess(duel) })
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_DuelProcess(duel)
    }
}

/// Returns a pointer to the message buffer, and writes the current message length in the provided pointer.
///
/// Use this to read messages from the core; subsequent calls invalidate previous buffers, so making copies
/// is recommended.
///
/// # Safety
/// The `length` pointer must be valid and mutable. The returned pointer points to internal core memory
/// and must not be mutated or freed manually.
pub unsafe fn OCG_DuelGetMessage(duel: OCG_Duel, length: *mut u32) -> *mut c_void {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| unsafe { backend.OCG_DuelGetMessage(duel, length) })
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_DuelGetMessage(duel, length)
    }
}

/// Sets the next player response for the duel simulation.
/// Subsequent calls overwrite previous responses if not processed.
/// The contents of the provided `response` buffer are copied internally, assuming it contains `length` bytes.
///
/// # Safety
/// The `response` pointer must point to a valid block of memory at least `length` bytes wide.
pub unsafe fn OCG_DuelSetResponse(duel: OCG_Duel, response: *const c_void, length: u32) {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| unsafe {
            backend.OCG_DuelSetResponse(duel, response, length)
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_DuelSetResponse(duel, response, length);
    }
}

/// Returns the number of cards in the specified zone.
///
/// # Safety
/// `duel` must point to an active, initialized duel instance.
pub unsafe fn OCG_DuelQueryCount(duel: OCG_Duel, team: u8, location: u32) -> u32 {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| unsafe {
            backend.OCG_DuelQueryCount(duel, team, location)
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_DuelQueryCount(duel, team, location)
    }
}

/// Returns a pointer to a buffer with the FIRST card found in the duel that matches the query.
/// The size of the buffer is written to `length` if the pointer is not null.
/// Subsequent calls invalidate previous queries.
///
/// # Safety
/// `query` must be a valid pointer to an initialized `OCG_QueryInfo` instance.
pub unsafe fn OCG_DuelQuery(
    duel: OCG_Duel,
    length: *mut u32,
    query: *const OCG_QueryInfo,
) -> *mut c_void {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| unsafe { backend.OCG_DuelQuery(duel, length, query) })
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_DuelQuery(duel, length, query)
    }
}

/// Returns a pointer to a buffer containing card counts for every zone in the game.
/// The size of the buffer is written to `length` if the pointer is not null.
/// Subsequent calls invalidate previous queries.
///
/// # Safety
/// `duel` must be a valid, live handle.
pub unsafe fn OCG_DuelQueryField(duel: OCG_Duel, length: *mut u32) -> *mut c_void {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| unsafe { backend.OCG_DuelQueryField(duel, length) })
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_DuelQueryField(duel, length)
    }
}

/// Returns a pointer to a buffer with ALL the cards matching the query.
/// The size of the buffer is written to `length` if the pointer is not null.
/// Subsequent calls invalidate previous queries.
///
/// # Safety
/// `query` must point to a valid `OCG_QueryInfo` layout.
pub unsafe fn OCG_DuelQueryLocation(
    duel: OCG_Duel,
    length: *mut u32,
    query: *const OCG_QueryInfo,
) -> *mut c_void {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| unsafe {
            backend.OCG_DuelQueryLocation(duel, length, query)
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_DuelQueryLocation(duel, length, query)
    }
}

/// Deallocates the duel instance created by [`OCG_CreateDuel()`].
///
/// # Safety
/// `duel` must be a valid pointer that was explicitly generated by `OCG_CreateDuel` and
/// has not been destroyed yet.
pub unsafe fn OCG_DestroyDuel(duel: OCG_Duel) {
    #[cfg(target_arch = "wasm32")]
    {
        crate::wasm::with_backend(|backend| unsafe { backend.OCG_DestroyDuel(duel) });
    }

    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        crate::native::OCG_DestroyDuel(duel);
    }
}
