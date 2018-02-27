const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const SCREEN_SIZE: usize = WIDTH * HEIGHT;

pub struct Lcd {
    screen: [bool; SCREEN_SIZE]
}

impl Lcd {
    pub fn new() -> Lcd {
        Lcd {
            screen: [false; SCREEN_SIZE]
        }
    }
}
