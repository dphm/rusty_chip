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
        self.current = self.current.saturating_sub(1);
    }

    pub fn set(&mut self, value: u8) {
        self.current = value;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_current_value() {
        let val: u8 = 42;
        let t = Timer::new(val);
        assert_eq!(val, t.current);
    }

    #[test]
    fn init_active() {
        let t = Timer::new(42);
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
    fn tick_active_decrements_current() {
        let mut t = Timer::new(42);
        let current = t.current;
        t.tick();

        assert_eq!(current - 1, t.current);
        assert!(t.active());
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
