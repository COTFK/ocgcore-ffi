#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

mod backend;
mod native;
mod types;
mod wasm;

use std::ffi::c_void;

use crate::backend::OCGCoreBackend;

use types::OCG_Duel;
use types::OCG_DuelOptions;
use types::OCG_NewCardInfo;

pub struct OCGCore {
    backend: Box<dyn OCGCoreBackend + Send + Sync>,
}

impl OCGCore {
    pub async fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            use crate::wasm::WasmCore;
            Self {
                backend: Box::new(WasmCore::new().await),
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use crate::native::NativeCore;
            Self {
                backend: Box::new(NativeCore {}),
            }
        }
    }
}

impl OCGCore {
    pub fn OCG_GetVersion(&self, major: &mut i32, minor: &mut i32) {
        self.backend.OCG_GetVersion(major, minor);
    }
    pub fn OCG_CreateDuel(
        &self,
        out_ocg_duel: *mut OCG_Duel,
        options_ptr: *const OCG_DuelOptions,
    ) -> i32 {
        self.backend.OCG_CreateDuel(out_ocg_duel, options_ptr)
    }
    pub fn OCG_DestroyDuel(&self, ocg_duel: OCG_Duel) {
        self.backend.OCG_DestroyDuel(ocg_duel)
    }
    pub fn OCG_DuelNewCard(&self, ocg_duel: OCG_Duel, info_ptr: *const OCG_NewCardInfo) {
        self.backend.OCG_DuelNewCard(ocg_duel, info_ptr)
    }
    pub fn OCG_StartDuel(&self, ocg_duel: OCG_Duel) {
        self.backend.OCG_StartDuel(ocg_duel)
    }
    pub fn OCG_DuelProcess(&self, ocg_duel: OCG_Duel) -> i32 {
        self.backend.OCG_DuelProcess(ocg_duel)
    }
    pub fn OCG_DuelGetMessage(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        self.backend.OCG_DuelGetMessage(ocg_duel, length)
    }
    pub fn OCG_DuelSetResponse(&self, ocg_duel: OCG_Duel, buffer: *const c_void, length: u32) {
        self.backend.OCG_DuelSetResponse(ocg_duel, buffer, length)
    }
    pub fn OCG_LoadScript(
        &self,
        ocg_duel: OCG_Duel,
        buffer: *const std::ffi::c_char,
        length: u32,
        name: *const std::ffi::c_char,
    ) -> i32 {
        self.backend.OCG_LoadScript(ocg_duel, buffer, length, name)
    }
    pub fn OCG_DuelQueryCount(&self, ocg_duel: OCG_Duel, team: u8, loc: u32) -> u32 {
        self.backend.OCG_DuelQueryCount(ocg_duel, team, loc)
    }
    pub fn OCG_DuelQuery(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const types::OCG_QueryInfo,
    ) -> *mut c_void {
        self.backend.OCG_DuelQuery(ocg_duel, length, info_ptr)
    }
    pub fn OCG_DuelQueryField(&self, ocg_duel: OCG_Duel, length: *mut u32) -> *mut c_void {
        self.backend.OCG_DuelQueryField(ocg_duel, length)
    }
    pub fn OCG_DuelQueryLocation(
        &self,
        ocg_duel: OCG_Duel,
        length: *mut u32,
        info_ptr: *const types::OCG_QueryInfo,
    ) -> *mut c_void {
        self.backend
            .OCG_DuelQueryLocation(ocg_duel, length, info_ptr)
    }
}
