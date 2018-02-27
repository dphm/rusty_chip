#[allow(dead_code)]

mod cpu;
mod memory;

use cpu::Cpu;
use memory::Memory;

fn main() {
    let mut memory = Memory::new();
    let mut cpu = Cpu::new(&mut memory);

    loop {
        if cpu.exit { return; }
        cpu.step();
    }
}
