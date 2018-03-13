use std::ops::Range;
use memory::Memory;

use {Address, Byte};

#[test]
fn load_data_with_len_equal_range() {
    let data: Vec<Byte> = vec![0xA, 0xB, 0xC, 0xD, 0xE];
    let range: Range<Address> = 0x0..0x5;
    let mut memory = Memory::new();

    memory.load(&data, range);
    assert_eq!(0xA, memory[0x0]);
    assert_eq!(0xB, memory[0x1]);
    assert_eq!(0xC, memory[0x2]);
    assert_eq!(0xD, memory[0x3]);
    assert_eq!(0xE, memory[0x4]);
}

#[test]
fn load_data_with_len_less_than_range() {
    let data: Vec<Byte> = vec![0xA, 0xB, 0xC];
    let range: Range<Address> = 0x0..0x5;
    let mut memory = Memory::new();

    memory.load(&data, range);
    assert_eq!(0xA, memory[0x0]);
    assert_eq!(0xB, memory[0x1]);
    assert_eq!(0xC, memory[0x2]);
}

#[test]
#[should_panic(expected = "Data length")]
fn load_data_with_len_greater_than_range_panics() {
    let data: Vec<Byte> = vec![0xA, 0xB, 0xC, 0xD, 0xE, 0xF];
    let range: Range<Address> = 0x0..0x5;
    let mut memory = Memory::new();

    memory.load(&data, range);
}
