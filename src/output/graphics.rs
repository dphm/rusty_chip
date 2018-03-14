pub const SPRITE_WIDTH_PIXELS: usize = 8;
pub const SCREEN_WIDTH_PIXELS: usize = 64;
pub const SCREEN_WIDTH_SPRITES: usize = SCREEN_WIDTH_PIXELS / SPRITE_WIDTH_PIXELS;
pub const SCREEN_HEIGHT_PIXELS: usize = 32;
const SCREEN_SIZE_PIXELS: usize = SCREEN_WIDTH_PIXELS * SCREEN_HEIGHT_PIXELS;

pub trait GraphicsOutput {
    fn display(&self, Vec<bool>);
}
