extern crate rusty_chip;

fn main() {
    let rom = rusty_chip::load_rom("rom" ,"logo.ch8");
    let mut cpu = rusty_chip::init_cpu(rom);

    loop {
        if cpu.exit { break; }
        cpu.step();
    }
}
