extern crate rusty_chip;

use rusty_chip::*;
use cpu::Cpu;
use output::graphics::Display;

use std::io::{BufReader, Read};
use std::fs::File;
use std::path::Path;

fn main() {
    let display = Display::new();
    let rom = load_rom("rom" ,"logo.ch8");
    let mut cpu = Cpu::new(&rom, display);

    loop {
        if cpu.exit { break; }
        cpu.step();
    }
}

pub fn load_rom(directory: &str, filename: &str) -> Vec<u8> {
    let path = Path::new(directory).join(filename);
    let mut file = File::open(path).expect("Unable to open file");
    read_bytes(&mut file)
}

fn read_bytes(file: &mut File) -> Vec<u8> {
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).expect("Unable to read file");
    contents
}
