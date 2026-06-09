//! Callback handler registry for WebAssembly backend.

use crate::types::OCG_DataReader;
use crate::types::OCG_DataReaderDone;
use crate::types::OCG_LogHandler;
use crate::types::OCG_ScriptReader;

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

/// Global storage that maps active Wasm heap addresses to their respective Rust handlers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallbackRegistry {
    pub card_readers: HashMap<usize, OCG_DataReader>,
    pub card_done_callbacks: HashMap<usize, OCG_DataReaderDone>,
    pub script_readers: HashMap<usize, OCG_ScriptReader>,
    pub log_handlers: HashMap<usize, OCG_LogHandler>,
    pub duel_payloads: HashMap<usize, [usize; 4]>,
}

pub static CALLBACK_REGISTRY: Lazy<Mutex<CallbackRegistry>> = Lazy::new(|| {
    Mutex::new(CallbackRegistry {
        card_readers: HashMap::new(),
        card_done_callbacks: HashMap::new(),
        script_readers: HashMap::new(),
        log_handlers: HashMap::new(),
        duel_payloads: HashMap::new(),
    })
});
