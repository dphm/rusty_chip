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

type Byte = u8;
type Address = usize;

pub fn init_cpu(rom: Vec<Byte>) -> Cpu {
    Cpu::new(&rom)
}

pub fn load_rom(filename: &str) -> Result<Vec<Byte>, Error> {
    let path = Path::new(filename);
    let mut file = File::open(path).expect("Unable to open file");
    read_bytes(&mut file)
}

fn read_bytes(file: &mut File) -> Result<Vec<Byte>, Error> {
    let mut buffer = Vec::new();
    match file.read_to_end(&mut buffer) {
        Ok(_) => Ok(buffer),
        Err(e) => Err(e)
    }
}

#[cfg(test)]
mod tests;
