//! Native FFI for `ocgcore`.
//!
//! This is the backend that will be used for any target that is not
//! `wasm32-unknown-unknown` - for that see [crate::wasm::WasmBackend].
//!
//! This module should only handle raw C FFI with `ocgcore` - no additional logic!

use std::ffi::c_char;
use std::ffi::c_void;

use super::types::OCG_Duel;
use super::types::OCG_DuelOptions;
use super::types::OCG_NewCardInfo;
use super::types::OCG_QueryInfo;

unsafe extern "C" {
    pub unsafe fn OCG_GetVersion(major: *mut i32, minor: *mut i32);
    pub unsafe fn OCG_CreateDuel(
        out_ocg_duel: *mut OCG_Duel,
        options_ptr: *const OCG_DuelOptions,
    ) -> i32;
    pub unsafe fn OCG_DestroyDuel(ocg_duel: OCG_Duel);
    pub unsafe fn OCG_DuelNewCard(ocg_duel: OCG_Duel, info_ptr: *const OCG_NewCardInfo);
    pub unsafe fn OCG_StartDuel(ocg_duel: OCG_Duel);
    pub unsafe fn OCG_DuelProcess(ocg_duel: OCG_Duel) -> i32;
    pub unsafe fn OCG_DuelGetMessage(ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void;
    pub unsafe fn OCG_DuelSetResponse(ocg_duel: OCG_Duel, buffer: *const c_void, length: u32);
    pub unsafe fn OCG_LoadScript(
        ocg_duel: OCG_Duel,
        buffer: *const c_char,
        length: u32,
        name: *const c_char,
    ) -> i32;
    pub unsafe fn OCG_DuelQueryCount(ocg_duel: OCG_Duel, team: u8, loc: u32) -> u32;
    pub unsafe fn OCG_DuelQuery(
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void;
    pub unsafe fn OCG_DuelQueryLocation(
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void;
    pub unsafe fn OCG_DuelQueryField(ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void;
}
