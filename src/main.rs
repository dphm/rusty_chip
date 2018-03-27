extern crate rusty_chip;

use rusty_chip::output::graphics::GraphicsOutput;

pub struct Display {}

impl GraphicsOutput for Display {
    fn display(&self, data: Vec<bool>) {
        let lines = data.iter().enumerate()
            .fold(String::new(), |mut acc, (i, bit)| {
                if (i % rusty_chip::output::graphics::SCREEN_WIDTH) == 0 {
                    acc.push_str(&format!("\n{:02} ", i / 64));
                }

                let c = match *bit {
                    true => "  ",
                    false => "▓▓︎"
                };

                acc.push_str(c);
                acc
            });
        println!("{}", lines)
    }
}

fn main() {
    let display = Display {};
    let rom = rusty_chip::load_rom("rom" ,"logo.ch8");
    let mut cpu = rusty_chip::init_cpu(&rom, &display);

    loop {
        if cpu.exit { break; }
        cpu.step();
        if cpu.draw {
            let data = cpu.display_data();
            display.display(data);
        }
    }
}
