#![feature(proc_macro, wasm_custom_section, wasm_import_module)]
#![feature(range_contains)]

extern crate wasm_bindgen;
use wasm_bindgen::prelude::*;

pub mod cpu;
pub mod output;

type Byte = u8;
type Address = usize;

#[wasm_bindgen]
extern {
    #[wasm_bindgen(js_namespace = Math)]
    fn random() -> f64;
}

#[wasm_bindgen]
pub struct WasmCpu {
    cpu: cpu::Cpu
}

#[wasm_bindgen]
impl WasmCpu {
    pub fn new() -> WasmCpu {
        let display = output::graphics::Display::new();

        WasmCpu {
            cpu: cpu::Cpu::new(display)
        }
    }

    pub fn load_rom(&mut self, rom: &[Byte]) {
        self.cpu.load_rom(rom);
    }

    pub fn exit(&self) -> bool {
        self.cpu.exit
    }
}
