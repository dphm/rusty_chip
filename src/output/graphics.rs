pub const SPRITE_WIDTH: usize = 8;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const SCREEN_WIDTH_SPRITES: usize = SCREEN_WIDTH / SPRITE_WIDTH;
const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

use Address;

pub struct Display {
    redraw: bool,
    memory: [bool; SCREEN_SIZE]
}

impl Display {
    pub fn new() -> Display {
        Display {
            redraw: false,
            memory: [false; SCREEN_SIZE]
        }
    }

    pub fn read_pixel(&self, x: Address, y: Address) -> bool {
        let x = x % SCREEN_WIDTH;
        let y = y % SCREEN_HEIGHT;
        self.memory[y * SCREEN_WIDTH + x]
    }

    pub fn update_pixel(&mut self, x: Address, y: Address, val: bool) -> bool {
        let x = x % SCREEN_WIDTH;
        let y = y % SCREEN_HEIGHT;
        let old = self.read_pixel(x, y);
        let collision = old & val;
        let new = old ^ val;
        if new != old {
            self.memory[y * SCREEN_WIDTH + x] = new;
            self.redraw = true;
        }
        collision
    }

    pub fn clear(&mut self) {
        self.memory = [false; SCREEN_SIZE];
    }

    pub fn draw(&mut self) {
        if !self.redraw { return; }

        let lines = self.memory.iter().enumerate()
            .fold(String::new(), |mut acc, (i, bit)| {
                if (i % SCREEN_WIDTH) == 0 {
                    acc.push_str(&format!("\n{:02} ", i / 64));
                }

                let c = match *bit {
                    true => "  ",
                    false => "▓▓︎"
                };

                acc.push_str(c);
                acc
            });
        println!("{}", lines);
        self.redraw = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_display() {
        let d = Display::new();
        assert!(!d.redraw);
        assert!(d.memory.iter().all(|bit| *bit == false));
        assert_eq!(SCREEN_SIZE, d.memory.len());
    }

    #[test]
    fn read_pixel_value() {
        let mut d = Display::new();
        let x = 8;
        let y = 30;
        let i = y * SCREEN_WIDTH + x;

        assert!(!d.read_pixel(x, y));

        d.memory[i] = true;
        assert!(d.read_pixel(x, y));
    }

    #[test]
    fn read_pixel_value_wrap() {
        let mut d = Display::new();
        let x = 64;
        let y = 30;
        let i = y * SCREEN_WIDTH + 0;

        assert!(!d.read_pixel(x, y));

        d.memory[i] = true;
        assert!(d.read_pixel(x, y));
    }

    #[test]
    fn update_pixel_false_xor_false() {
        let mut d = Display::new();
        let x = 8;
        let y = 30;
        let i = y * SCREEN_WIDTH + x;

        let collision = d.update_pixel(x, y, false);
        assert!(!d.memory[i], "false XOR false should set the pixel to false");
        assert!(!collision, "false AND false should not be a collision");
        assert!(!d.redraw, "false -> false should not redraw");
    }

    #[test]
    fn update_pixel_false_xor_true() {
        let mut d = Display::new();
        let x = 8;
        let y = 30;
        let i = y * SCREEN_WIDTH + x;

        let collision = d.update_pixel(x, y, true);
        assert!(d.memory[i], "false XOR true should set the pixel to true");
        assert!(!collision, "false AND true should not be a collision");
        assert!(d.redraw, "false -> true should redraw");
    }

    #[test]
    fn update_pixel_true_xor_false() {
        let mut d = Display::new();
        let x = 8;
        let y = 30;
        let i = y * SCREEN_WIDTH + x;

        d.memory[i] = true;
        let collision = d.update_pixel(x, y, false);
        assert!(d.memory[i], "true XOR false should set the pixel to true");
        assert!(!collision, "true AND false should not be a collision");
        assert!(!d.redraw, "true -> true should not redraw");
    }

    #[test]
    fn update_pixel_true_xor_true() {
        let mut d = Display::new();
        let x = 8;
        let y = 30;
        let i = y * SCREEN_WIDTH + x;

        d.memory[i] = true;
        let collision = d.update_pixel(x, y, true);
        assert!(!d.memory[i], "true XOR true should set the pixel to false");
        assert!(collision, "true AND true should be a collision");
        assert!(d.redraw, "true -> false should redraw");
    }

    #[test]
    fn update_pixel_wrap() {
        let mut d = Display::new();
        let x = 64;
        let y = 30;
        let i = y * SCREEN_WIDTH + 0;

        d.memory[i] = true;
        let collision = d.update_pixel(x, y, true);
        assert!(!d.memory[i], "true XOR true should set the pixel to false");
        assert!(collision, "true AND true should be a collision");
        assert!(d.redraw, "true -> false should redraw");
    }

    #[test]
    fn clear_display() {
        let mut d = Display::new();
        d.memory = [true; SCREEN_SIZE];

        d.clear();
        assert!(d.memory.iter().all(|pixel| *pixel == false), "clear should set all pixels to false");
    }

    #[test]
    fn draw_resets_redraw_to_false() {
        let mut d = Display::new();
        d.redraw = true;

        d.draw();
        assert!(!d.redraw, "draw should reset redraw to false");
    }
}
