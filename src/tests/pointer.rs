use std::ops::Range;
use pointer::Pointer;

use Address;

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