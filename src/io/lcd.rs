const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const SCREEN_SIZE: usize = WIDTH * HEIGHT;

type Byte = u8;

use std::fmt::{Debug, Display, Formatter, Result};

pub struct Lcd {
    screen: [bool; SCREEN_SIZE]
}

impl Lcd {
    pub fn new() -> Lcd {
        Lcd {
            screen: [false; SCREEN_SIZE]
        }
    }

    pub fn draw(&mut self, point: (Byte, Byte), byte: Byte) -> bool {
        false
    }
}

impl Display for Lcd {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let strs = self.screen.iter()
            .map(|on| {
                match *on {
                    true => "■",
                    false => "□"
                }
            });

        let lines = strs.enumerate()
            .fold(String::new(), |mut acc, (i, s)| {
                if i % WIDTH == 0 {
                    acc.push_str("\n");
                }

                acc.push_str(&s);
                acc
            });

        write!(f, "{}", lines)
    }
}

impl Debug for Lcd {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self)
    }
}
