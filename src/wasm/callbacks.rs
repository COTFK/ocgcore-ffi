use std::ffi::c_char;
use std::ffi::c_int;
use std::ffi::c_void;

use crate::types::OCG_CardData;
use crate::types::OCG_DataReader;
use crate::types::OCG_DataReaderDone;
use crate::types::OCG_Duel;
use crate::types::OCG_LogHandler;
use crate::types::OCG_ScriptReader;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallbackRegistry {
    pub card_readers: HashMap<usize, OCG_DataReader>,
    pub card_done_callbacks: HashMap<usize, OCG_DataReaderDone>,
    pub script_readers: HashMap<usize, OCG_ScriptReader>,
    pub log_handlers: HashMap<usize, OCG_LogHandler>,
}

pub static CALLBACK_REGISTRY: Lazy<Mutex<CallbackRegistry>> = Lazy::new(|| {
    Mutex::new(CallbackRegistry {
        card_readers: HashMap::new(),
        card_done_callbacks: HashMap::new(),
        script_readers: HashMap::new(),
        log_handlers: HashMap::new(),
    })
});

fn get_callback<T, F>(accessor: F) -> Option<T>
where
    T: Clone,
    F: FnOnce(&CallbackRegistry) -> Option<&Option<T>>,
{
    let registry = CALLBACK_REGISTRY.lock().unwrap();
    accessor(&registry).cloned().flatten()
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wasm_log_trampoline(
    payload: *mut c_void,
    string: *const c_char,
    log_type: c_int,
) {
    if let Some(callback) = get_callback(|reg| reg.log_handlers.get(&(payload as usize))) {
        unsafe {
            callback(payload, string, log_type);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wasm_script_reader_trampoline(
    payload: *mut c_void,
    duel: OCG_Duel,
    name: *const c_char,
) -> i32 {
    get_callback(|reg| reg.script_readers.get(&(payload as usize)))
        .map_or_else(|| 0, |callback| unsafe { callback(payload, duel, name) })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wasm_card_reader_trampoline(
    payload: *mut c_void,
    code: u32,
    data: *mut OCG_CardData,
) {
    if let Some(callback) = get_callback(|reg| reg.card_readers.get(&(payload as usize))) {
        unsafe {
            callback(payload, code, data);
        }
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wasm_card_done_trampoline(payload: *mut c_void, data: *mut OCG_CardData) {
    if let Some(callback) = get_callback(|reg| reg.card_done_callbacks.get(&(payload as usize))) {
        unsafe {
            callback(payload, data);
        }
    }
}
