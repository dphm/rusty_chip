pub const SPRITE_WIDTH: usize = 8;
pub const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub trait GraphicsOutput {
    fn display(&self, [bool; SCREEN_SIZE]);
}
