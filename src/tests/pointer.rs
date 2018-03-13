use std::ops::Range;
use pointer::Pointer;

use Address;

#[test]
fn current_defaults_to_range_start() {
    let range: Range<Address> = 0x100..0xF00;
    let p = Pointer::new(&range);
    assert_eq!(0x100, p.current);
}

#[test]
fn move_forward_adds_2_to_current() {
    let range: Range<Address> = 0x100..0xF00;
    let mut p = Pointer::new(&range);

    p.move_forward();
    assert_eq!(0x102, p.current);

    p.move_forward();
    assert_eq!(0x104, p.current);
}

#[test]
fn move_backward_subtracts_2_from_current() {
    let range: Range<Address> = 0x100..0xF00;
    let mut p = Pointer::new(&range);
    p.current = 0xF00;

    p.move_backward();
    assert_eq!(0xEFE, p.current);

    p.move_backward();
    assert_eq!(0xEFC, p.current);
}

#[test]
fn set_current() {
    let range: Range<Address> = 0x100..0xF00;
    let mut p = Pointer::new(&range);
    let addr = 0xABC;

    p.set(addr);
    assert_eq!(addr, p.current);
}