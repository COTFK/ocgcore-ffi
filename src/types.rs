//! Raw types used by the `ocgcore` functions.
//!
//! These should be nearly-identical to their C++ equivalents.
//!
//! # Thread Safety
//! Types containing raw pointers—such as [`OCG_CardData`] (which contains a `*mut u16`)
//! and [`OCG_DuelOptions`] (which contains multiple `*mut c_void` payloads) are implicitly
//! marked as `!Send` and `!Sync` by the Rust compiler.
//!
//! If you intend to use these types across thread boundaries,
//! you must provide your own explicit synchronization primitives (e.g., `Mutex`)
//! or safely implement `Send` and `Sync` using a custom marker layout.

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_int, c_void};

/// Represents a player and their starting state (LP, cards in hand, draws per turn).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OCG_Player {
    pub starting_lp: u32,
    pub starting_draw_count: u32,
    pub draw_count_per_turn: u32,
}

/// Non-gameplay related card information - card code, original ATK, DEF, etc.
///
/// Generally you want to instantiate this for every card in the duel,
/// using data from a static database of sorts, whenever `ocgcore` calls the OCG_DataReader callback.
///
/// # Thread Safety
/// Because this type contains a `*mut u16`, the Rust compiler implicitly marks this type
/// as `!Send` and `!Sync`.
///
/// If you intend to use this type across thread boundaries within a safe wrapper
/// crate, you must provide your own explicit synchronization primitives (e.g., `Mutex`)
/// or safely implement `Send` and `Sync` using a custom marker layout.
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

/// A handle to a duel instance.
///
/// You should store this and keep it alive for the duration of the duel,
/// as most functions require this handle to exist to work properly.
pub type OCG_Duel = *mut c_void;

/// Card reader callback - `ocgcore` will call this *before* a card is added to the duel.
pub type OCG_DataReader =
    Option<unsafe extern "C" fn(payload: *mut c_void, code: u32, data: *mut OCG_CardData)>;

/// Card reader finish callback - `ocgcore` will call this *after* a card is added to the duel.
pub type OCG_DataReaderDone =
    Option<unsafe extern "C" fn(payload: *mut c_void, data: *mut OCG_CardData)>;

/// Script reader callback - `ocgcore` will call this *before* a card is added to the duel.
pub type OCG_ScriptReader = Option<
    unsafe extern "C" fn(payload: *mut c_void, duel: OCG_Duel, name: *const c_char) -> c_int,
>;

/// Log callback - `ocgcore` will call this when it wants to output log data.
pub type OCG_LogHandler =
    Option<unsafe extern "C" fn(payload: *mut c_void, string: *const c_char, log_type: c_int)>;

/// Duel settings - rulesets, players, and the callbacks used during duel instantiation.
///
/// # Thread Safety
/// Because this type contains multiple `*mut c_void` payloads, the Rust compiler implicitly
/// marks this type as `!Send` and `!Sync`.
///
/// If you intend to use this type across thread boundaries within a safe wrapper
/// crate, you must provide your own explicit synchronization primitives (e.g., `Mutex`)
/// or safely implement `Send` and `Sync` using a custom marker layout.
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

/// Gameplay-related card information - owner, location, index, etc.
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

/// A query passed to one of the query functions; the fields act as filters.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct OCG_QueryInfo {
    pub flags: u32,
    pub con: u8,
    pub loc: u32,
    pub seq: u32,
    pub overlay_seq: u32,
}
