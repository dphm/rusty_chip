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
