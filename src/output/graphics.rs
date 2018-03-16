pub const SPRITE_WIDTH: usize = 8;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const SCREEN_WIDTH_SPRITES: usize = SCREEN_WIDTH / SPRITE_WIDTH;
const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub trait GraphicsOutput {
    fn display(&self, Vec<bool>);
}
