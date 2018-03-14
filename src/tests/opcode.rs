use opcode::Opcode;

use {Address, Byte};

#[test]
fn mask_first_hex_digit() {
    let codes = [
        0x0000, 0x1111, 0x2222, 0x3333,
        0x4444, 0x5555, 0x6666, 0x7777,
        0x8888, 0x9999, 0xAAAA, 0xBBBB,
        0xCCCC, 0xDDDD, 0xEEEE, 0xFFFF
    ];

    for i in 0x0..0x10 {
        let opcode = Opcode::new(codes[i]);
        assert_eq!(i as Byte, opcode.first_hex_digit());
    }
}

#[test]
fn mask_x() {
    let codes = [
        0x0000, 0x0100, 0x0200, 0x0300,
        0x0400, 0x0500, 0x0600, 0x0700,
        0x0800, 0x0900, 0x0A00, 0x0B00,
        0x0C00, 0x0D00, 0x0E00, 0x0F00
    ];

    for i in 0x0..0x10 {
        let opcode = Opcode::new(codes[i]);
        assert_eq!(i as Address, opcode.x());
    }
}

#[test]
fn mask_y() {
    let codes = [
        0x0000, 0x0010, 0x0020, 0x0030,
        0x0040, 0x0050, 0x0060, 0x0070,
        0x0080, 0x0090, 0x00A0, 0x00B0,
        0x00C0, 0x00D0, 0x00E0, 0x00F0
    ];

    for i in 0x0..0x10 {
        let opcode = Opcode::new(codes[i]);
        assert_eq!(i as Address, opcode.y());
    }
}

#[test]
fn mask_nnn() {
    let code = 0xABCD;
    let opcode = Opcode::new(code);
    assert_eq!(0xBCD as Address, opcode.nnn());
}

#[test]
fn mask_kk() {
    let code = 0xABCD;
    let opcode = Opcode::new(code);
    assert_eq!(0xCD as Byte, opcode.kk());
}

#[test]
fn mask_k() {
    let code = 0xABCD;
    let opcode = Opcode::new(code);
    assert_eq!(0xD as usize, opcode.k());
}