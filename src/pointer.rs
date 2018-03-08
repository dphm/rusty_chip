use std::fmt::{self, Debug};
use std::ops::Range;

type Address = usize;

pub struct Pointer<'a> {
    pub current: Address,
    range: &'a Range<Address>,
    step_size: usize,
}

impl<'a> Pointer<'a> {
    pub fn new(range: &Range<Address>) -> Pointer {
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

impl<'a> Debug for Pointer<'a> {
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

    #[test]
    fn current_defaults_to_range_start() {
        let range: Range<Address> = 0x100..0xF00;
        let p = Pointer::new(&range);
        assert_eq!(0x100, p.current);
    }

    #[test]
    fn step_size_defaults_to_2() {
        let range: Range<Address> = 0x100..0xF00;
        let p = Pointer::new(&range);
        assert_eq!(2, p.step_size);
    }

    #[test]
    fn move_forward_adds_step_size_to_current() {
        let range: Range<Address> = 0x100..0xF00;
        let mut p = Pointer::new(&range);

        p.move_forward();
        assert_eq!(p.range.start + p.step_size, p.current);

        p.move_forward();
        assert_eq!(p.range.start + p.step_size * 2, p.current);
    }

    #[test]
    fn move_backward_subtracts_step_size_from_current() {
        let range: Range<Address> = 0x100..0xF00;
        let mut p = Pointer::new(&range);
        p.current = p.range.end;

        p.move_backward();
        assert_eq!(p.range.end - p.step_size, p.current);

        p.move_backward();
        assert_eq!(p.range.end - p.step_size * 2, p.current);
    }

    #[test]
    fn set_current() {
        let range: Range<Address> = 0x100..0xF00;
        let mut p = Pointer::new(&range);
        let addr = 0xABC;

        p.set(addr);
        assert_eq!(addr, p.current);
    }
}
