extern crate rusty_chip;

use rusty_chip::*;
use output::graphics;

fn main() {
    let display = graphics::Display::new();
    let rom = load_rom("rom" ,"logo.ch8");
    let mut cpu = init_cpu(&rom, display);

    loop {
        if cpu.exit { break; }
        cpu.step();
    }
}
