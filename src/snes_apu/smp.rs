// TODO: This will panic on certain opcodes. The ideas I have for cpu_lang
// will probably fix it; we'll see. For now tho, I'm gonna leave it as-is;
// I can imagine fixing it trivially might lead to performance problems.
// I'll deal with it after the port is finished.

// TODO: This is a placeholder before I start generalizing traits
// from the old code.
use super::apu::Apu;

pub struct Smp<'a> {
    emulator: &'a mut Apu,

    reg_pc: u16,
    reg_a: u8,
    reg_x: u8,
    reg_y: u8,
    reg_sp: u8,

    psw_c: bool,
    psw_z: bool,
    psw_h: bool,
    psw_p: bool,
    psw_v: bool,
    psw_n: bool,
    // TODO: Look up some more behavior for I and B. Can't seem to find much
    // but some instructions DO set them.
    psw_i: bool,
    psw_b: bool,

    cycle_count: i32
}

impl<'a> Smp<'a> {
    pub fn new(emulator: &'a mut Apu) -> Smp<'a> {
        let mut ret = Smp {
            emulator: emulator,

            reg_pc: 0,
            reg_a: 0,
            reg_x: 0,
            reg_y: 0,
            reg_sp: 0,

            psw_c: false,
            psw_z: false,
            psw_h: false,
            psw_p: false,
            psw_v: false,
            psw_n: false,
            psw_i: false,
            psw_b: false,

            cycle_count: 0
        };
        ret.reset();
        ret
    }

    pub fn reset(&mut self) {
        self.reg_pc = 0xffc0;
        self.reg_a = 0;
        self.reg_x = 0;
        self.reg_y = 0;
        self.reg_sp = 0xef;
        self.set_psw(0x02);
    }

    pub fn set_reg_ya(&mut self, value: u16) {
        self.reg_a = (value & 0xff) as u8;
        self.reg_y = ((value >> 8) & 0xff) as u8;
    }

    pub fn get_reg_ya(&self) -> u16 {
        ((self.reg_y as u16) << 8) | (self.reg_a as u16)
    }

    pub fn set_psw(&mut self, value: u8) {
        self.psw_c = (value & 0x01) != 0;
        self.psw_z = (value & 0x02) != 0;
        self.psw_h = (value & 0x08) != 0;
        self.psw_p = (value & 0x20) != 0;
        self.psw_v = (value & 0x40) != 0;
        self.psw_n = (value & 0x80) != 0;
    }

    pub fn get_psw(&self) -> u8 {
        ((if self.psw_n { 1 } else { 0 }) << 7) |
        ((if self.psw_v { 1 } else { 0 }) << 6) |
        ((if self.psw_p { 1 } else { 0 }) << 5) |
        ((if self.psw_h { 1 } else { 0 }) << 3) |
        ((if self.psw_z { 1 } else { 0 }) << 1) |
        (if self.psw_c { 1 } else { 0 })
    }

    fn is_negative(value: u32) -> bool {
        (value & 0x80) != 0
    }

    fn cycles(&mut self, num_cycles: i32) {
        self.emulator.cpu_cycles_callback(num_cycles);
        self.cycle_count += num_cycles;
    }

    fn run(&mut self, target_cycles: i32) -> i32 {
        0 // TODO
    }
}
