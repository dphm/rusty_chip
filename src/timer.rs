#[derive(Debug)]
pub struct Timer {
    pub current: u8
}

impl Timer {
    pub fn new(initial: u8) -> Timer {
        Timer {
            current: initial
        }
    }

    pub fn active(&self) -> bool {
        self.current > 0
    }

    pub fn tick(&mut self) {
        self.current = self.current.saturating_sub(1);
    }

    pub fn set(&mut self, value: u8) {
        self.current = value;
    }
}
