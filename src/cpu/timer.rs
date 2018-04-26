#[derive(Clone, Debug, PartialEq)]
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
        if !self.active() { return; }
        let updated = self.current.saturating_sub(1);
        self.set(updated);
    }

    pub fn set(&mut self, val: u8) {
        self.current = val;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_active() {
        let val = 42;
        let t = Timer::new(val);
        assert_eq!(val, t.current);
        assert!(t.active());
    }

    #[test]
    fn init_inactive() {
        let t = Timer::new(0);
        assert!(!t.active());
    }

    #[test]
    fn tick_inactive_no_op() {
        let mut t = Timer::new(0);
        t.tick();

        assert_eq!(0, t.current);
        assert!(!t.active());
    }

    #[test]
    fn tick_active_decrements_by_one() {
        let mut t = Timer::new(60);
        
        t.tick();
        assert_eq!(59, t.current);

        t.tick();
        assert_eq!(58, t.current);
    }

    #[test]
    fn tick_deactivates_at_zero() {
        let mut t = Timer::new(1);

        t.tick();
        assert_eq!(0, t.current);
        assert!(!t.active());
    }

    #[test]
    fn set_current_value() {
        let val: u8 = 42;
        let mut t = Timer::new(24);
        t.set(val);

        assert_eq!(val, t.current);
    }
}
