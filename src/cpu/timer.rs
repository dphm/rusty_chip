use std::time::{Duration, Instant};

#[derive(Clone, Debug, PartialEq)]
pub struct Timer {
    pub current: u8,
    last_set: Instant,
    interval: Duration
}

impl Timer {
    pub fn new(initial: u8, rate: u32) -> Timer {
        Timer {
            current: initial,
            last_set: Instant::now(),
            interval: Duration::new(1, 0) / rate
        }
    }

    pub fn active(&self) -> bool {
        self.current > 0
    }

    pub fn tick(&mut self) {
        let duration_passed = Instant::now().duration_since(self.last_set);
        if duration_passed >= self.interval {
            let intervals_passed = (duration_passed.subsec_nanos() / self.interval.subsec_nanos()) as u8;
            let updated = self.current.saturating_sub(intervals_passed);
            self.set(updated);
        }
    }

    pub fn set(&mut self, value: u8) {
        self.current = value;
        self.last_set = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use std::thread::sleep;
    use super::*;

    #[test]
    fn init_active() {
        let val = 42;
        let t = Timer::new(val, 60);
        assert_eq!(val, t.current);
        assert!(t.active());
    }

    #[test]
    fn init_inactive() {
        let t = Timer::new(0, 60);
        assert!(!t.active());
    }

    #[test]
    fn tick_inactive_no_op() {
        let mut t = Timer::new(0, 60);
        let first_set = t.last_set;
        t.tick();

        assert_eq!(0, t.current);
        assert_eq!(first_set, t.last_set);
        assert!(!t.active());
    }

    #[test]
    fn tick_active_decrements_current_at_rate() {
        let rate = 60;
        let mut t = Timer::new(60, rate);
        let quarter_second = t.interval * rate / 4;

        sleep(quarter_second);
        t.tick();

        assert_eq!(45, t.current);

        sleep(quarter_second);
        t.tick();

        assert_eq!(30, t.current);

        sleep(quarter_second);
        t.tick();

        assert_eq!(15, t.current);

        sleep(quarter_second);
        t.tick();

        assert_eq!(0, t.current);
    }

    #[test]
    fn tick_deactivates_at_zero() {
        let mut t = Timer::new(1, 60);

        sleep(t.interval);
        t.tick();

        assert_eq!(0, t.current);
        assert!(!t.active());
    }

    #[test]
    fn set_current_value() {
        let val: u8 = 42;
        let mut t = Timer::new(24, 60);
        let first_set = t.last_set;
        t.set(val);

        assert_eq!(val, t.current);
        assert!(t.last_set > first_set);
    }    
}
