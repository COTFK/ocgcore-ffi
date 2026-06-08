//! Compilation script for `src/ocgcore`.
//! 
//! For native targets, it compiles it normally via `[cc]`.
//! For WebAssembly, Emscripten is used to compile the library to a JS+WebAssembly bundle.

use std::env;
use std::path::PathBuf;
use std::process::Command;

const OCGCORE_ROOT: &str = "src/ocgcore";

const EMSDK_NOT_FOUND: &str =
    "========================================================================
    Error: EMSDK is not set.

    To cross-compile 'ocgcore-ffi' for WebAssembly targets,
    please install the Emscripten SDK and set the environment.

    You can follow the official instructions here:
        https://emscripten.org/docs/getting_started/downloads.html        \n\
    ========================================================================";

fn main() {
    println!("cargo:rerun-if-changed=src/ocgcore");

    let is_wasm = env::var("TARGET").is_ok_and(|target| target.contains("wasm32"));
    if is_wasm {
        // Get path to Emscripten SDK - panics if not found
        let emsdk_path =
            env::var("EMSDK").map_or_else(|_| panic!("{EMSDK_NOT_FOUND}"), PathBuf::from);
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
        let ocgcore_path = PathBuf::from(OCGCORE_ROOT);

        let mut build = Command::new(emsdk_path.join("upstream/emscripten/em++"));
        build.args([
            "-Os",
            "-g0",
            "-flto",
            "--closure",
            "2",
            "-std=c++17",
            "-fexceptions",
            "-sMODULARIZE=1",
            "-sEXPORT_ES6=1",
            "-sWASM=0",
            "-sFILESYSTEM=0",
            "-sEXPORT_NAME=ocgcore",
            "-sEXPORTED_FUNCTIONS=['_OCG_GetVersion','_OCG_CreateDuel','_OCG_DestroyDuel','_OCG_DuelNewCard','_OCG_StartDuel','_OCG_DuelProcess','_OCG_DuelGetMessage','_OCG_DuelSetResponse','_OCG_LoadScript','_OCG_DuelQueryCount','_OCG_DuelQuery','_OCG_DuelQueryLocation','_OCG_DuelQueryField','_malloc','_free']",
            "-sEXPORTED_RUNTIME_METHODS=['wasmMemory','addFunction']",
            "-sALLOW_MEMORY_GROWTH=1",
            "-sALLOW_TABLE_GROWTH=1",
            "-sENVIRONMENT=web",
            "-I",
            ocgcore_path.join("lua/src").to_str().unwrap(),
            "-I",
            ocgcore_path.to_str().unwrap(),
            "-DMAKE_LIB",
            "-o",
            manifest_dir.join("ocgcore.js").to_str().unwrap(),
            ocgcore_path.join("card.cpp").to_str().unwrap(),
            ocgcore_path.join("duel.cpp").to_str().unwrap(),
            ocgcore_path.join("effect.cpp").to_str().unwrap(),
            ocgcore_path.join("field.cpp").to_str().unwrap(),
            ocgcore_path.join("interpreter.cpp").to_str().unwrap(),
            ocgcore_path.join("libcard.cpp").to_str().unwrap(),
            ocgcore_path.join("libdebug.cpp").to_str().unwrap(),
            ocgcore_path.join("libduel.cpp").to_str().unwrap(),
            ocgcore_path.join("libeffect.cpp").to_str().unwrap(),
            ocgcore_path.join("libgroup.cpp").to_str().unwrap(),
            ocgcore_path.join("ocgapi.cpp").to_str().unwrap(),
            ocgcore_path.join("operations.cpp").to_str().unwrap(),
            ocgcore_path.join("playerop.cpp").to_str().unwrap(),
            ocgcore_path.join("processor.cpp").to_str().unwrap(),
            ocgcore_path.join("processor_visit.cpp").to_str().unwrap(),
            ocgcore_path.join("scriptlib.cpp").to_str().unwrap(),
            ocgcore_path.join("lua/src/onelua.c").to_str().unwrap(),
        ]);

        let status = build.status().expect("Failed to execute em++");

        if !status.success() {
            panic!("ocgcore compilation failed");
        }
    }

    if !is_wasm {
        let core_root = PathBuf::from(OCGCORE_ROOT);
        let mut lua_build = cc::Build::new();
        lua_build
            .cpp(true)
            .define("MAKE_LIB", None)
            .file(core_root.join("lua/src/onelua.c"))
            .include(core_root.join("lua/src"));

        let mut ocgcore_build = cc::Build::new();
        ocgcore_build
            .cpp(true)
            .include(core_root.join("lua/src"))
            .include(&core_root);

        let entries =
            std::fs::read_dir(&core_root).expect("Failed to read native ocgcore directory");
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "cpp") {
                ocgcore_build.file(path);
            }
        }
        lua_build.compile("luacore");
        ocgcore_build.compile("ocgcore");
    }
}
