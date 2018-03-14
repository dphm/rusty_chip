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
}

pub fn return_from_subroutine(cpu: &mut Cpu, _opcode: &Opcode) {
    let addr = cpu.stack_pop();
    cpu.jump(addr);
}

pub fn jump_addr(cpu: &mut Cpu, opcode: &Opcode) {
    let addr = opcode.nnn();
    cpu.jump(addr);
}

pub fn call_addr(cpu: &mut Cpu, opcode: &Opcode) {
    let addr = opcode.nnn();
    cpu.stack_push();
    cpu.jump(addr);
}

pub fn skip_equal_vx_byte(cpu: &mut Cpu, opcode: &Opcode) {
    let vx = cpu.read_register(opcode.x());
    let byte = opcode.kk();
    skip_if(cpu, vx == byte);
}

pub fn skip_not_equal_vx_byte(cpu: &mut Cpu, opcode: &Opcode) {
    let vx = cpu.read_register(opcode.x());
    let byte = opcode.kk();
    skip_if(cpu, vx != byte);
}

pub fn skip_equal_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let vx = cpu.read_register(opcode.x());
    let vy = cpu.read_register(opcode.y());
    skip_if(cpu, vx == vy);
}

pub fn load_vx_byte(cpu: &mut Cpu, opcode: &Opcode) {
    cpu.load_register(opcode.x(), opcode.kk());
}

pub fn add_vx_byte(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let byte = opcode.kk();
    let vx = cpu.read_register(x);
    cpu.load_register(x, vx.wrapping_add(byte));
}

pub fn load_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vy = cpu.read_register(opcode.y());
    cpu.load_register(x, vy);
}

pub fn or_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(opcode.y());
    cpu.load_register(x, vx | vy);
}

pub fn and_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(opcode.y());
    cpu.load_register(x, vx & vy);
}

pub fn xor_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(opcode.y());
    cpu.load_register(x, vx ^ vy);
}

pub fn add_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(opcode.y());
    let (result, carry) = vx.overflowing_add(vy);
    cpu.load_register(x, result);
    cpu.load_flag(carry);
}

pub fn sub_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(opcode.y());
    let (result, borrow) = vx.overflowing_sub(vy);
    cpu.load_register(x, result);
    cpu.load_flag(!borrow);
}

pub fn shr_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x);
    let (result, overflow) = vx.overflowing_shr(1);
    cpu.load_register(x, result);
    cpu.load_flag(overflow);
}

pub fn subn_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(opcode.y());
    let (result, borrow) = vy.overflowing_sub(vx);
    cpu.load_register(x, result);
    cpu.load_flag(!borrow);
}

pub fn shl_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x);
    let (result, overflow) = vx.overflowing_shl(1);
    cpu.load_register(x, result);
    cpu.load_flag(overflow);
}

pub fn skip_not_equal_vx_vy(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let vx = cpu.read_register(x);
    let vy = cpu.read_register(opcode.y());
    skip_if(cpu, vx != vy);
}

pub fn load_i_addr(cpu: &mut Cpu, opcode: &Opcode) {
    cpu.load_i(opcode.nnn());
}

pub fn jump_v0_addr(cpu: &mut Cpu, opcode: &Opcode) {
    let v0 = cpu.read_register(0x0) as Address;
    cpu.jump(opcode.nnn() + v0);
}

pub fn rand_vx_byte(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let byte = opcode.kk();
    let random_byte: Byte = rand::thread_rng().gen_range(0x0, 0xFF);
    cpu.load_register(x, random_byte & byte);
}

pub fn draw_vx_vy_n(cpu: &mut Cpu, opcode: &Opcode) {
    let vx = cpu.read_register(opcode.x()) as Address;
    let vy = cpu.read_register(opcode.y()) as Address;
    let n = opcode.k();

    let sprite_bytes = cpu.read_image_bytes(n);
    let byte_x = vx / graphics::SPRITE_WIDTH_PIXELS;
    let offset_x = vx % graphics::SPRITE_WIDTH_PIXELS;

    for offset_y in 0..n {
        let byte_y = (vy + offset_y) * graphics::SCREEN_WIDTH_SPRITES;
        let from_addr = byte_y + byte_x;
        let sprite_byte = sprite_bytes[offset_y];

        let mut to_bytes = Vec::new();
        if offset_x == 0 {
            to_bytes.push(sprite_byte);
        } else {
            let shift_r = offset_x as u32;
            let shift_l = (graphics::SCREEN_WIDTH_PIXELS - offset_x) as u32;
            to_bytes.push(sprite_byte.wrapping_shr(shift_r));
            to_bytes.push(sprite_byte.wrapping_shl(shift_l));
        }

        let collision = cpu.load_image_bytes(from_addr, to_bytes);
        cpu.load_flag(collision);
    }

    println!("{}", cpu);
}

pub fn skip_key_pressed_vx(_cpu: &mut Cpu, _opcode: &Opcode) {}
pub fn skip_key_not_pressed_vx(_cpu: &mut Cpu, _opcode: &Opcode) {}

pub fn load_vx_dt(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let dt = cpu.read_delay_timer();
    cpu.load_register(x, dt);
}

pub fn load_vx_key(_cpu: &mut Cpu, _opcode: &Opcode) {}

pub fn load_dt_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let vx = cpu.read_register(opcode.x());
    cpu.load_delay_timer(vx);
}

pub fn load_st_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let vx = cpu.read_register(opcode.x());
    cpu.load_sound_timer(vx);
}

pub fn add_i_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let i = cpu.read_i();
    let vx = cpu.read_register(opcode.x()) as Address;
    cpu.load_i(i.wrapping_add(vx));
}

pub fn load_font_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let vx = cpu.read_register(opcode.x()) as Address;
    cpu.load_i(vx * font::SPRITE_HEIGHT);
}

pub fn load_bcd_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let vx = cpu.read_register(opcode.x());
    let i = cpu.read_i();
    let bytes = vec![
        vx / 100,
        vx % 100 / 10,
        vx % 10
    ];
    cpu.load_image_bytes(i, bytes);
}

pub fn load_through_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let i = cpu.read_i();
    let bytes = (0..x + 1).map(|r| cpu.read_register(r)).collect();
    cpu.load_image_bytes(i, bytes);
}

pub fn read_through_vx(cpu: &mut Cpu, opcode: &Opcode) {
    let x = opcode.x();
    let bytes = cpu.read_image_bytes(x + 1);
    for (r, byte) in bytes.iter().enumerate() {
        cpu.load_register(r, *byte);
    }
}

fn skip_if(cpu: &mut Cpu, p: bool) {
    if p { cpu.skip(); }
}