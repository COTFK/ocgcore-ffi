#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_int, c_void};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OCG_Player {
    pub starting_lp: u32,
    pub starting_draw_count: u32,
    pub draw_count_per_turn: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OCG_CardData {
    pub code: u32,
    pub alias: u32,
    pub setcodes: *mut u16,
    pub r#type: u32,
    pub level: u32,
    pub attribute: u32,
    pub race: u64,
    pub attack: i32,
    pub defense: i32,
    pub lscale: u32,
    pub rscale: u32,
    pub link_marker: u32,
}

pub type OCG_Duel = *mut c_void;

pub type OCG_DataReader =
    Option<unsafe extern "C" fn(payload: *mut c_void, code: u32, data: *mut OCG_CardData)>;

pub type OCG_DataReaderDone =
    Option<unsafe extern "C" fn(payload: *mut c_void, data: *mut OCG_CardData)>;

pub type OCG_ScriptReader = Option<
    unsafe extern "C" fn(payload: *mut c_void, duel: OCG_Duel, name: *const c_char) -> c_int,
>;

pub type OCG_LogHandler =
    Option<unsafe extern "C" fn(payload: *mut c_void, string: *const c_char, log_type: c_int)>;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct OCG_DuelOptions {
    pub seed: [u64; 4],
    pub flags: u64,
    pub team1: OCG_Player,
    pub team2: OCG_Player,
    pub cardReader: OCG_DataReader,
    pub payload1: *mut c_void,
    pub scriptReader: OCG_ScriptReader,
    pub payload2: *mut c_void,
    pub logHandler: OCG_LogHandler,
    pub payload3: *mut c_void,
    pub cardReaderDone: OCG_DataReaderDone,
    pub payload4: *mut c_void,
    pub enableUnsafeLibraries: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OCG_NewCardInfo {
    pub team: u8,
    pub duelist: u8,
    pub code: u32,
    pub con: u8,
    pub loc: u32,
    pub seq: u32,
    pub pos: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OCG_QueryInfo {
    pub flags: u32,
    pub con: u8,
    pub loc: u32,
    pub seq: u32,
    pub overlay_seq: u32,
}