use std::ffi::c_void;
use std::ffi::c_char;

use crate::types::OCG_Duel;
use crate::types::OCG_DuelOptions;
use crate::types::OCG_NewCardInfo;
use crate::types::OCG_QueryInfo;

pub trait OCGCoreBackend {
    fn OCG_GetVersion(&self, major: &mut i32, minor: &mut i32);
    fn OCG_CreateDuel(&self, out: *mut OCG_Duel, opts: *const OCG_DuelOptions) -> i32;
    fn OCG_DestroyDuel(&self, ocg_duel: OCG_Duel);
    fn OCG_DuelNewCard(&self, ocg_duel: OCG_Duel, info_ptr: *const OCG_NewCardInfo);
    fn OCG_StartDuel(&self, ocg_duel: OCG_Duel);
    fn OCG_DuelProcess(&self, ocg_duel: OCG_Duel) -> i32;
    fn OCG_DuelGetMessage(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void;
    fn OCG_DuelSetResponse(&self, ocg_duel: OCG_Duel, buffer: *const c_void, length: u32);
    fn OCG_LoadScript(
        &self,
        ocg_duel: OCG_Duel,
        buffer: *const c_char,
        length: u32,
        name: *const c_char,
    ) -> i32;
    fn OCG_DuelQueryCount(&self, ocg_duel: OCG_Duel, team: u8, loc: u32) -> u32;
    fn OCG_DuelQuery(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void;
    fn OCG_DuelQueryLocation(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const OCG_QueryInfo,
    ) -> *mut c_void;
    fn OCG_DuelQueryField(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void;
}
