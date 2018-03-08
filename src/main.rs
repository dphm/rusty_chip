#![feature(range_contains)]

mod cpu;
mod memory;
mod opcode;
mod timer;
mod pointer;
mod output;

use std::io::{Error, Read};
use std::fs::File;
use std::path::Path;

use cpu::Cpu;
use memory::Memory;

fn main() {
    let rom = read_bytes("rom/logo.ch8").expect("Unable to load ROM");
    let mut memory = Memory::new();
    let mut cpu = Cpu::new(&mut memory, &rom);

    loop {
        if cpu.exit { break; }
        cpu.step();
    }
}

fn read_bytes(filename: &str) -> Result<Vec<u8>, Error> {
    let path = Path::new(filename);
    let mut file = File::open(path).expect("Unable to open file");
    let mut buffer = Vec::new();
    match file.read_to_end(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(e) => Err(e)
    }
}
