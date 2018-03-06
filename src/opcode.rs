type Address = usize;
type Byte = u8;

pub struct Opcode {
    code: u16
}

impl Opcode {
    pub fn new(code: u16) -> Opcode {
        Opcode { code }
    }

    pub fn from_bytes(bytes: (Byte, Byte)) -> Opcode {
        Opcode {
            code: ((bytes.0 as u16) << 8) | (bytes.1 as u16)
        }
    }

    pub fn first_hex_digit(&self) -> Byte {
        ((self.code & 0xF000) >> 12) as Byte
    }

    pub fn x(&self) -> Address {
        ((self.code & 0x0F00) >> 8) as Address
    }

    pub fn y(&self) -> Address {
        ((self.code & 0x00F0) >> 4) as Address
    }

    pub fn nnn(&self) -> Address {
        (self.code & 0x0FFF) as Address
    }

    pub fn kk(&self) -> Byte {
        (self.code & 0xFF) as Byte
    }

    pub fn k(&self) -> Byte {
        (self.code & 0xF) as Byte
    }
}
