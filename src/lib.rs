#![feature(range_contains)]

mod cpu;
mod memory;
mod output;

use std::io::{BufReader, Read};
use std::fs::File;
use std::path::Path;

use cpu::Cpu;

type Byte = u8;
type Address = usize;

pub fn init_cpu(rom: Vec<Byte>) -> Cpu {
    Cpu::new(&rom)
}

pub fn load_rom(directory: &str, filename: &str) -> Vec<Byte> {
    let path = Path::new(directory).join(filename);
    let mut file = File::open(path).expect("Unable to open file");
    read_bytes(&mut file)
}

fn read_bytes(file: &mut File) -> Vec<Byte> {
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).expect("Unable to read file");
    contents
}
