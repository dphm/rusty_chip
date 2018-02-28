#[derive(Debug)]
pub struct Timer {
    pub current: u8,
    pub active: bool
}

impl Timer {
    pub fn new(initial: u8) -> Timer {
        Timer {
            current: initial,
            active: true
        }
    }

    pub fn tick(&mut self) {
        if !self.active { return; }

        self.current -= 1;
        if self.current == 0 {
            self.active = false;
        }
    }

    pub fn set(&mut self, value: u8) {
        self.current = value;
    }
}
