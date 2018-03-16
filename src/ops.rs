extern crate rand;

use self::rand::Rng;

use cpu::Cpu;
use opcode::Opcode;
use output::{font, graphics};

use {Address, Byte};

pub fn no_op(_cpu: &mut Cpu, _opcode: &Opcode) {}

pub fn unknown(_cpu: &mut Cpu, opcode: &Opcode) {
    panic!("Unknown opcode {}", opcode);
}

pub fn clear_display(cpu: &mut Cpu, _opcode: &Opcode) {
    cpu.clear_display();
    println!("\tCLS");
}

pub fn return_from_subroutine(cpu: &mut Cpu, _opcode: &Opcode) {
    let addr = cpu.stack_pop();
    cpu.jump(addr);
    println!("\tRTN => {:x}", addr);
}

pub fn jump_addr(cpu: &mut Cpu, opcode: &Opcode) {
    let addr = opcode.nnn();
    cpu.jump(addr);
    println!("\tJP {:x}", addr);
}

pub fn call_addr(cpu: &mut Cpu, opcode: &Opcode) {
    let addr = opcode.nnn();
    cpu.stack_push();
    cpu.jump(addr);
    println!("\tCALL {:x}", addr);
}

pub fn skip_equal_vx_byte(cpu: &mut Cpu, opcode: &Opcode) {
    let vx = cpu.read_register(opcode.x());
    let byte = opcode.kk();
    skip_if(cpu, vx == byte);
    println!("\tSE vx: {:x}, byte: {:x}", vx, byte);
}

pub fn skip_not_equal_vx_byte(cpu: &mut Cpu, opcode: &Opcode) {
    let vx = cpu.read_register(opcode.x());
    let byte = opcode.kk();
    skip_if(cpu, vx != byte);
    println!("\tSNE vx: {:x}, byte: {:x}", vx, byte);
}

pub fn skip_equal_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let vx = cpu.read_register(opcode.x());
    let vy = cpu.read_register(opcode.y());
    skip_if(cpu, vx == vy);
    println!("\tSE vx: {:x}, vy: {:x}", vx, vy);
}

pub fn load_vx_byte(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    cpu.load_register(x, opcode.kk());
    println!("\tLD V{:x}, byte: {:x} => {:x}", x, opcode.kk(), cpu.read_register(x));
}

pub fn add_vx_byte(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let byte = opcode.kk();
    let vx = cpu.read_register(x);
    cpu.load_register(x, vx.wrapping_add(byte));
    println!("\tADD V{:x}: {:x}, byte: {:x} => {:x}", x, vx, byte, cpu.read_register(x));
}

pub fn load_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let y = opcode.y();
    let vy = cpu.read_register(y);
    cpu.load_register(x, vy);
    println!("\tLD V{:x}, V{:x}: {:x} => {:x}", x, y, vy, cpu.read_register(x));
}

pub fn or_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let y = opcode.y();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(y);
    cpu.load_register(x, vx | vy);
    println!("\tOR V{:x}: {:x}, V{:x}: {:x} => {:x}", x, vx, y, vy, cpu.read_register(x));
}

pub fn and_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let y = opcode.y();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(y);
    cpu.load_register(x, vx & vy);
    println!("\tAND V{:x}: {:x}, V{:x}: {:x} => {:x}", x, vx, y, vy, cpu.read_register(x));
}

pub fn xor_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let y = opcode.y();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(y);
    cpu.load_register(x, vx ^ vy);
    println!("\tXOR V{:x}: {:x}, V{:x}: {:x} => {:x}", x, vx, y, vy, cpu.read_register(x));
}

pub fn add_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let y = opcode.y();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(y);
    let (result, carry) = vx.overflowing_add(vy);
    cpu.load_register(x, result);
    cpu.load_flag(carry);
    println!("\tADD V{:x}: {:x}, V{:x}: {:x} => ({:x}, {})", x, vx, y, vy, cpu.read_register(x), cpu.read_register(0xF));
}

pub fn sub_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let y = opcode.y();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(y);
    let (result, borrow) = vx.overflowing_sub(vy);
    cpu.load_register(x, result);
    cpu.load_flag(!borrow);
    println!("\tSUB V{:x}: {:x}, V{:x}: {:x} => ({:x}, {})", x, vx, y, vy, cpu.read_register(x), cpu.read_register(0xF));
}

pub fn shr_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let y = opcode.y();
    let vy = cpu.read_register(y);
    let (result, overflow) = vy.overflowing_shr(1);
    cpu.load_register(x, result);
    cpu.load_flag(overflow);
    println!("\tSHR V{:x}, V{:x}: {:x} => ({:x}, {})", x, y, vy, cpu.read_register(x), cpu.read_register(0xF));
}

pub fn subn_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let y = opcode.y();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(y);
    let (result, borrow) = vy.overflowing_sub(vx);
    cpu.load_register(x, result);
    cpu.load_flag(!borrow);
    println!("\tSUBN V{:x}: {:x}, V{:x}: {:x} => ({:x}, {})", x, vx, y, vy, cpu.read_register(x), cpu.read_register(0xF));
}

pub fn shl_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let y = opcode.y();
    let vy = cpu.read_register(y);
    let (result, overflow) = vy.overflowing_shl(1);
    cpu.load_register(x, result);
    cpu.load_flag(overflow);
    println!("\tSHL V{:x}, V{:x}: {:x} => ({:x}, {})", x, y, vy, cpu.read_register(x), cpu.read_register(0xF));
}

pub fn skip_not_equal_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let y = opcode.y();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(y);
    skip_if(cpu, vx != vy);
    println!("\tSNE V{:x}: {:x}, V{:x}: {:x}", x, vx, y, vy);
}

pub fn load_i_addr(cpu: &mut Cpu, opcode: &Opcode) {
    let addr = opcode.nnn();
    cpu.load_i(addr);
    println!("\tLD I, {:x} => {:x}", addr, cpu.read_i());
}

pub fn jump_v0_addr(cpu: &mut Cpu, opcode: &Opcode) {
    let v0 = cpu.read_register(0x0) as Address;
    let addr = opcode.nnn();
    cpu.jump(addr + v0);
    println!("\tJP V0: {:x}, {:x}", v0, addr);
}

pub fn rand_vx_byte(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let byte = opcode.kk();
    let random_byte: Byte = rand::thread_rng().gen_range(0x0, 0xFF);
    cpu.load_register(x, random_byte & byte);
    println!("\tRND V{:x} => {:x}", x, cpu.read_register(x));
}

pub fn draw_vx_vy_n(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let y = opcode.y();
    let vx = cpu.read_register(x) as Address;
    let vy = cpu.read_register(y) as Address;
    let n = opcode.k();

    let sprite_bytes = cpu.read_image_bytes(n);
    let byte_x = (vx / graphics::SCREEN_WIDTH_SPRITES) % graphics::SCREEN_WIDTH_SPRITES;
    let pixel_x = vx % graphics::SPRITE_WIDTH;

    for sprite_y in 0..n {
        let byte_y = (vy + sprite_y) % graphics::SCREEN_HEIGHT;
        let sprite_byte = sprite_bytes[sprite_y];
        let mut collision = match pixel_x {
            0 => cpu.load_image_byte(byte_x, byte_y, sprite_byte),
            _ => {
                let shift_r = pixel_x as u32;
                let left = sprite_byte.wrapping_shr(shift_r);

                let shift_l = (graphics::SPRITE_WIDTH - pixel_x) as u32;
                let right = sprite_byte.wrapping_shl(shift_l);
                let right_x = (byte_x + 1) % graphics::SCREEN_WIDTH_SPRITES;

                cpu.load_image_byte(byte_x, byte_y, left) &
                    cpu.load_image_byte(right_x, byte_y, right)
            }
        };

        cpu.load_flag(collision);
    }

    println!("\tDRW V{:x}: {:x}, V{:x}: {:x}, {:?}", x, vx, y, vy, sprite_bytes);
    print!("   ");
    for i in 0..graphics::SCREEN_WIDTH_SPRITES {
        print!("{}               ", i);
    }
    println!("{}", cpu);
}

pub fn skip_key_pressed_vx(_cpu: &mut Cpu, _opcode: &Opcode) {}
pub fn skip_key_not_pressed_vx(_cpu: &mut Cpu, _opcode: &Opcode) {}

pub fn load_vx_dt(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let dt = cpu.read_delay_timer();
    cpu.load_register(x, dt);
    println!("\tLD V{:x}, DT: {:x} => {:x}", x, dt, cpu.read_register(x));
}

pub fn load_vx_key(_cpu: &mut Cpu, _opcode: &Opcode) {}

pub fn load_dt_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x);
    cpu.load_delay_timer(vx);
    println!("\tLD DT, V{:x}: {:x} => {:x}", x, vx, cpu.read_delay_timer());
}

pub fn load_st_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x);
    cpu.load_sound_timer(vx);
    println!("\tLD ST, V{:x}: {:x} => {:x}", x, vx, cpu.read_sound_timer());
}

pub fn add_i_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let i = cpu.read_i();
    let x = opcode.x();
    let vx = cpu.read_register(x) as Address;
    cpu.load_i(i.wrapping_add(vx));
    println!("\tADD I: {:x}, V{:x}: {:x} => {:x}", i, x, vx, cpu.read_i());
}

pub fn load_i_vx_font(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x) as Address;
    cpu.load_i(vx * font::SPRITE_HEIGHT);
    println!("\tLD I, FONT V{:x}: {:x} => {:x}", x, vx, cpu.read_i());
}

pub fn load_bcd_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x);
    let bytes = vec![
        vx / 100,
        vx % 100 / 10,
        vx % 10
    ];
    cpu.load_image_bytes(bytes);
    println!("\tLD BCD V{:x}: {:x} ({}) => {:?}", x, vx, vx, cpu.read_image_bytes(3));
}

pub fn load_through_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let i = cpu.read_i();
    let bytes = (0..x + 1).map(|r| cpu.read_register(r)).collect();
    cpu.load_image_bytes(bytes);
    cpu.load_i(i + x + 1);

    let register_bytes: Vec<Byte> = (0..x + 1).map(|r| cpu.read_register(r)).collect();
    println!("\tLD [I], V{:x} [{:?}] => {:?}", x, register_bytes, cpu.read_image_bytes(x + 1));
}

pub fn read_through_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let i = cpu.read_i();
    let bytes = cpu.read_image_bytes(x + 1);
    for (r, byte) in bytes.iter().enumerate() {
        cpu.load_register(r, *byte);
    }
    cpu.load_i(i + x + 1);

    let register_bytes: Vec<Byte> = (0..x + 1).map(|r| cpu.read_register(r)).collect();
    println!("\tRD V{:x} [{:?}], [I] => {:?}", x, cpu.read_image_bytes(x + 1), register_bytes);
}

fn skip_if(cpu: &mut Cpu, p: bool) {
    if p { cpu.skip(); }
}
