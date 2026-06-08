//! `ocgcore` native backend.
//!
//! This is the backend that will be used for any target that is not
//! `wasm32-unknown-unknown` - for that see [crate::wasm::WasmBackend].
//!
//! This module should only handle raw C FFI with `ocgcore` - no additional logic!

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_int, c_void};

use crate::types::OCG_Duel;
use crate::types::OCG_DuelOptions;
use crate::types::OCG_NewCardInfo;
use crate::types::OCG_QueryInfo;

/// Native backend; routes directly to the C++ library via FFI.
/// 
/// The methods on this struct are identical to the C++ methods and provide no additional
/// abstraction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NativeBackend;

impl NativeBackend {
    pub fn OCG_GetVersion(&self, major: &mut i32, minor: &mut i32) {
        unsafe { ffi::OCG_GetVersion(major, minor) };
    }
    pub fn OCG_CreateDuel(
        &self,
        out_ocg_duel: *mut OCG_Duel,
        options_ptr: *const OCG_DuelOptions,
    ) -> i32 {
        unsafe { ffi::OCG_CreateDuel(out_ocg_duel, options_ptr) }
    }
    pub fn OCG_DestroyDuel(&self, ocg_duel: OCG_Duel) {
        unsafe { ffi::OCG_DestroyDuel(ocg_duel) }
    }
    pub fn OCG_DuelNewCard(&self, ocg_duel: OCG_Duel, info_ptr: *const OCG_NewCardInfo) {
        unsafe { ffi::OCG_DuelNewCard(ocg_duel, info_ptr) }
    }
    pub fn OCG_StartDuel(&self, ocg_duel: OCG_Duel) {
        unsafe { ffi::OCG_StartDuel(ocg_duel) }
    }
    pub fn OCG_DuelProcess(&self, ocg_duel: OCG_Duel) -> i32 {
        unsafe { ffi::OCG_DuelProcess(ocg_duel) }
    }
    pub fn OCG_DuelGetMessage(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        unsafe { ffi::OCG_DuelGetMessage(ocg_duel, length) }
    }
    pub fn OCG_DuelSetResponse(&self, ocg_duel: OCG_Duel, buffer: *const c_void, length: u32) {
        unsafe { ffi::OCG_DuelSetResponse(ocg_duel, buffer, length) }
    }
    pub fn OCG_LoadScript(
        &self,
        ocg_duel: OCG_Duel,
        buffer: *const c_char,
        length: u32,
        name: *const c_char,
    ) -> i32 {
        unsafe { ffi::OCG_LoadScript(ocg_duel, buffer, length, name) }
    }
    pub fn OCG_DuelQueryCount(&self, ocg_duel: OCG_Duel, team: u8, loc: u32) -> u32 {
        unsafe { ffi::OCG_DuelQueryCount(ocg_duel, team, loc) }
    }
    pub fn OCG_DuelQuery(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void {
        unsafe { ffi::OCG_DuelQuery(ocg_duel, length, info_ptr) }
    }
    pub fn OCG_DuelQueryLocation(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void {
        unsafe { ffi::OCG_DuelQueryLocation(ocg_duel, length, info_ptr) }
    }
    pub fn OCG_DuelQueryField(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        unsafe { ffi::OCG_DuelQueryField(ocg_duel, length) }
    }
}

mod ffi {
    use super::*;

    unsafe extern "C" {
        pub fn OCG_GetVersion(major: *mut c_int, minor: *mut c_int);
        pub fn OCG_CreateDuel(
            out_ocg_duel: *mut OCG_Duel,
            options_ptr: *const OCG_DuelOptions,
        ) -> c_int;
        pub fn OCG_DestroyDuel(ocg_duel: OCG_Duel);
        pub fn OCG_DuelNewCard(ocg_duel: OCG_Duel, info_ptr: *const OCG_NewCardInfo);
        pub fn OCG_StartDuel(ocg_duel: OCG_Duel);
        pub fn OCG_DuelProcess(ocg_duel: OCG_Duel) -> c_int;
        pub fn OCG_DuelGetMessage(ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void;
        pub fn OCG_DuelSetResponse(ocg_duel: OCG_Duel, buffer: *const c_void, length: u32);
        pub fn OCG_LoadScript(
            ocg_duel: OCG_Duel,
            buffer: *const c_char,
            length: u32,
            name: *const c_char,
        ) -> c_int;
        pub fn OCG_DuelQueryCount(ocg_duel: OCG_Duel, team: u8, loc: u32) -> u32;
        pub fn OCG_DuelQuery(
            ocg_duel: OCG_Duel,
            length: *mut u32,
            info_ptr: *const OCG_QueryInfo,
        ) -> *mut c_void;
        pub fn OCG_DuelQueryLocation(
            ocg_duel: OCG_Duel,
            length: *mut u32,
            info_ptr: *const OCG_QueryInfo,
        ) -> *mut c_void;
        pub fn OCG_DuelQueryField(ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void;
    }
}
