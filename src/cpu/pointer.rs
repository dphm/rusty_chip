use std::fmt::{self, Debug};
use std::ops::Range;

use Address;

pub struct Pointer {
    pub current: Address,
    range: Range<Address>,
    step_size: usize,
}

impl Pointer {
    pub fn new(range: Range<Address>) -> Pointer {
        Pointer {
            current: range.start,
            range,
            step_size: 2,
        }
    }

    pub fn move_forward(&mut self) {
        let next = self.current + self.step_size;
        if next >= self.range.end {
            panic!(
                "Pointer value greater than pointer range ({:x}..{:x})",
                self.range.start, self.range.end
            );
        } else {
            self.current = next;
        }
    }

    pub fn move_backward(&mut self) {
        let prev = self.current - self.step_size;
        if prev < self.range.start {
            panic!(
                "Pointer value less than pointer range ({:x}..{:x})",
                self.range.start, self.range.end
            );
        } else {
            self.current = prev;
        }
    }

    pub fn set(&mut self, addr: Address) {
        if !self.range.contains(addr) {
            panic!(
                "Address {:x} out of pointer range ({:x}..{:x})",
                addr, self.range.start, self.range.end
            );
        } else {
            self.current = addr;
        }
    }
}

impl Debug for Pointer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,
            "Pointer {{ current: {:x}, range: {:x}..{:x}, step_size: {} }}",
            self.current, self.range.start, self.range.end, self.step_size
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_RANGE: Range<Address> = 0x100..0xF00;

    #[test]
    fn current_defaults_to_range_start() {
        let p = Pointer::new(TEST_RANGE);
        assert_eq!(TEST_RANGE.start, p.current);
    }

    #[test]
    fn move_forward_adds_2_to_current() {
        let mut p = Pointer::new(TEST_RANGE);

        p.move_forward();
        assert_eq!(TEST_RANGE.start + 2, p.current);

        p.move_forward();
        assert_eq!(TEST_RANGE.start + 4, p.current);
    }

    #[test]
    fn move_backward_subtracts_2_from_current() {
        let mut p = Pointer::new(TEST_RANGE);
        p.current = TEST_RANGE.end;

        p.move_backward();
        assert_eq!(TEST_RANGE.end - 2, p.current);

        p.move_backward();
        assert_eq!(TEST_RANGE.end - 4, p.current);
    }

    #[test]
    fn set_current() {
        let mut p = Pointer::new(TEST_RANGE);
        let addr = 0xABC;

        p.set(addr);
        assert_eq!(addr, p.current);
    }
}
