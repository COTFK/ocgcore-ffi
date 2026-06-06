#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_int, c_void};

use crate::backend::OCGCoreBackend;
use crate::types::OCG_Duel;
use crate::types::OCG_DuelOptions;
use crate::types::OCG_NewCardInfo;
use crate::types::OCG_QueryInfo;

#[allow(dead_code)]
pub struct NativeCore;

impl OCGCoreBackend for NativeCore {
    fn OCG_GetVersion(&self, major: &mut i32, minor: &mut i32) {
        unsafe { _OCG_GetVersion(major, minor) };
    }
    fn OCG_CreateDuel(
        &self,
        out_ocg_duel: *mut OCG_Duel,
        options_ptr: *const OCG_DuelOptions,
    ) -> i32 {
        unsafe { _OCG_CreateDuel(out_ocg_duel, options_ptr) }
    }
    fn OCG_DestroyDuel(&self, ocg_duel: OCG_Duel) {
        unsafe { _OCG_DestroyDuel(ocg_duel) }
    }
    fn OCG_DuelNewCard(&self, ocg_duel: OCG_Duel, info_ptr: *const OCG_NewCardInfo) {
        unsafe { _OCG_DuelNewCard(ocg_duel, info_ptr) }
    }
    fn OCG_StartDuel(&self, ocg_duel: OCG_Duel) {
        unsafe { _OCG_StartDuel(ocg_duel) }
    }
    fn OCG_DuelProcess(&self, ocg_duel: OCG_Duel) -> i32 {
        unsafe { _OCG_DuelProcess(ocg_duel) }
    }
    fn OCG_DuelGetMessage(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        unsafe { _OCG_DuelGetMessage(ocg_duel, length) }
    }
    fn OCG_DuelSetResponse(&self, ocg_duel: OCG_Duel, buffer: *const c_void, length: u32) {
        unsafe { _OCG_DuelSetResponse(ocg_duel, buffer, length) }
    }
    fn OCG_LoadScript(
        &self,
        ocg_duel: OCG_Duel,
        buffer: *const c_char,
        length: u32,
        name: *const c_char,
    ) -> i32 {
        unsafe { _OCG_LoadScript(ocg_duel, buffer, length, name) }
    }
    fn OCG_DuelQueryCount(&self, ocg_duel: OCG_Duel, team: u8, loc: u32) -> u32 {
        unsafe { _OCG_DuelQueryCount(ocg_duel, team, loc) }
    }
    fn OCG_DuelQuery(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void {
        unsafe { _OCG_DuelQuery(ocg_duel, length, info_ptr) }
    }
    fn OCG_DuelQueryLocation(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void {
        unsafe { _OCG_DuelQueryLocation(ocg_duel, length, info_ptr) }
    }
    fn OCG_DuelQueryField(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        unsafe { _OCG_DuelQueryField(ocg_duel, length) }
    }
}

unsafe extern "C" {
    #[link_name = "OCG_GetVersion"]
    unsafe fn _OCG_GetVersion(major: *mut c_int, minor: *mut c_int);

    #[link_name = "OCG_CreateDuel"]
    unsafe fn _OCG_CreateDuel(
        out_ocg_duel: *mut OCG_Duel,
        options_ptr: *const OCG_DuelOptions,
    ) -> c_int;

    #[link_name = "OCG_DestroyDuel"]
    unsafe fn _OCG_DestroyDuel(ocg_duel: OCG_Duel);

    #[link_name = "OCG_DuelNewCard"]
    unsafe fn _OCG_DuelNewCard(ocg_duel: OCG_Duel, info_ptr: *const OCG_NewCardInfo);

    #[link_name = "OCG_StartDuel"]
    unsafe fn _OCG_StartDuel(ocg_duel: OCG_Duel);

    #[link_name = "OCG_DuelProcess"]
    unsafe fn _OCG_DuelProcess(ocg_duel: OCG_Duel) -> c_int;

    #[link_name = "OCG_DuelGetMessage"]
    unsafe fn _OCG_DuelGetMessage(ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void;

    #[link_name = "OCG_DuelSetResponse"]
    unsafe fn _OCG_DuelSetResponse(ocg_duel: OCG_Duel, buffer: *const c_void, length: u32);

    #[link_name = "OCG_LoadScript"]
    unsafe fn _OCG_LoadScript(
        ocg_duel: OCG_Duel,
        buffer: *const c_char,
        length: u32,
        name: *const c_char,
    ) -> c_int;

    #[link_name = "OCG_DuelQueryCount"]
    unsafe fn _OCG_DuelQueryCount(ocg_duel: OCG_Duel, team: u8, loc: u32) -> u32;

    #[link_name = "OCG_DuelQuery"]
    unsafe fn _OCG_DuelQuery(
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void;

    #[link_name = "OCG_DuelQueryLocation"]
    unsafe fn _OCG_DuelQueryLocation(
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void;

    #[link_name = "OCG_DuelQueryField"]
    unsafe fn _OCG_DuelQueryField(ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void;
}
