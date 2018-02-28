#[allow(dead_code)]

mod cpu;
mod memory;
mod io;

use std::io::{Error, Read};
use std::fs::File;
use std::path::Path;

use cpu::Cpu;
use memory::Memory;

fn main() {
    let rom = read_bytes("rom/logo.ch8").expect("Unable to load ROM");
    let mut memory = Memory::new(&rom);
    let mut cpu = Cpu::new(&mut memory);

    loop {
        if cpu.exit { return; }
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
