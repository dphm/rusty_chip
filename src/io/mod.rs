const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub mod font;

pub trait Input {
    fn up(&self) -> bool;
    fn down(&self) -> bool;
    fn left(&self) -> bool;
    fn right(&self) -> bool;
    fn hex_digit(&self) -> u8;
}

pub trait GraphicsOutput {
    fn display(&self, [bool; SCREEN_SIZE]);
}

pub trait SoundOutput {
    fn beep(&self, bool);
}
