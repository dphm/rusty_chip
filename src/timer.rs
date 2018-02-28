pub struct Timer {
    pub current: u16,
    pub active: bool
}

impl Timer {
    pub fn new(initial: u16) -> Timer {
        Timer {
            current: initial,
            active: true
        }
    }

    pub fn tick(&mut self) {
        self.current -= 1;
        if self.current == 0 {
            self.active = false;
        }
    }
}
