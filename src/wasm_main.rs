#![feature(proc_macro, wasm_custom_section, wasm_import_module)]

extern crate rusty_chip;
use rusty_chip::*;

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

use cpu::Cpu;
use output::graphics::Display;

#[wasm_bindgen]
pub struct WasmCpu {
    cpu: Cpu
}

#[wasm_bindgen]
impl WasmCpu {
    pub fn new() -> WasmCpu {
        let rom = Vec::new();
        let display = Display::new();
        WasmCpu {
            cpu: Cpu::new(&rom, display)
        }
    }
}

fn main() {}
