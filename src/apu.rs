use super::smp::Smp;
use super::dsp::dsp::Dsp;
use super::timer::Timer;
use super::spc::spc::{Spc, RAM_LEN, IPL_ROM_LEN};

static DEFAULT_IPL_ROM: [u8; IPL_ROM_LEN] = [
    0xcd, 0xef, 0xbd, 0xe8, 0x00, 0xc6, 0x1d, 0xd0,
    0xfc, 0x8f, 0xaa, 0xf4, 0x8f, 0xbb, 0xf5, 0x78,
    0xcc, 0xf4, 0xd0, 0xfb, 0x2f, 0x19, 0xeb, 0xf4,
    0xd0, 0xfc, 0x7e, 0xf4, 0xd0, 0x0b, 0xe4, 0xf5,
    0xcb, 0xf4, 0xd7, 0x00, 0xfc, 0xd0, 0xf3, 0xab,
    0x01, 0x10, 0xef, 0x7e, 0xf4, 0x10, 0xeb, 0xba,
    0xf6, 0xda, 0x00, 0xba, 0xf4, 0xc4, 0xf4, 0xdd,
    0x5d, 0xd0, 0xdb, 0x1f, 0x00, 0x00, 0xc0, 0xff,
];

pub struct Apu {
    ram: [u8; RAM_LEN],
    ipl_rom: [u8; IPL_ROM_LEN],

    pub smp: Smp,
    pub dsp: Dsp,

    timers: [Timer; 3],

    is_ipl_rom_enabled: bool,
    dsp_reg_address: u8
}

impl Apu {
    pub fn new() -> Apu {
        let mut apu = Apu {
            ram: [0; RAM_LEN],
            ipl_rom: DEFAULT_IPL_ROM,

            // temporary values that will be replaced
            smp: Smp::new(0 as _),
            dsp: Dsp::new(0 as _),

            timers: [Timer::new(256), Timer::new(256), Timer::new(32)],

            is_ipl_rom_enabled: true,
            dsp_reg_address: 0
        };
        let apu_ptr = &mut apu as *mut _;
        apu.smp = Smp::new(apu_ptr);
        apu.dsp = Dsp::new(apu_ptr);
        apu.dsp.init(apu_ptr);
        apu
    }

    pub fn from_spc(spc: &Spc) -> Apu {
        let mut ret = Apu::new();

        for i in 0..RAM_LEN {
            ret.ram[i] = spc.ram[i];
        }
        for i in 0..IPL_ROM_LEN {
            ret.ipl_rom[i] = spc.ipl_rom[i];
        }

        {
            let smp = &mut ret.smp;
            smp.reg_pc = spc.pc;
            smp.reg_a = spc.a;
            smp.reg_x = spc.x;
            smp.reg_y = spc.y;
            smp.set_psw(spc.psw);
            smp.reg_sp = spc.sp;
        }

        ret.dsp.set_state(spc);

        for i in 0..3 {
            let target = ret.ram[0xfa + i];
            ret.timers[i].set_target(target);
        }
        let control_reg = ret.ram[0xf1];
        ret.set_control_reg(control_reg);

        ret.dsp_reg_address = ret.ram[0xf2];

        ret
    }

    pub fn render(&mut self, left_buffer: &mut [i16], right_buffer: &mut [i16], num_samples: i32) {
        while self.dsp.output_buffer.get_sample_count() < num_samples {
            self.smp.run(num_samples * 64);
            self.dsp.flush();
        }

        self.dsp.output_buffer.read(left_buffer, right_buffer, num_samples);
    }

    pub fn cpu_cycles_callback(&mut self, num_cycles: i32) {
        self.dsp.cycles_callback(num_cycles);
        for timer in self.timers.iter_mut() {
            timer.cpu_cycles_callback(num_cycles);
        }
    }

    pub fn read_u8(&mut self, address: u32) -> u8 {
        let address = address & 0xffff;
        if address >= 0xf0 && address < 0x0100 {
            match address {
                0xf0 | 0xf1 => 0,

                0xf2 => self.dsp_reg_address,
                0xf3 => self.dsp.get_register(self.dsp_reg_address),

                0xfa ... 0xfc => 0,

                0xfd => self.timers[0].read_counter(),
                0xfe => self.timers[1].read_counter(),
                0xff => self.timers[2].read_counter(),

                _ => self.ram[address as usize]
            }
        } else if address >= 0xffc0 && self.is_ipl_rom_enabled {
            self.ipl_rom[(address - 0xffc0) as usize]
        } else {
            self.ram[address as usize]
        }
    }

    pub fn write_u8(&mut self, address: u32, value: u8) {
        let address = address & 0xffff;
        if address >= 0x00f0 && address < 0x0100 {
            match address {
                0xf0 => { self.set_test_reg(value); },
                0xf1 => { self.set_control_reg(value); },
                0xf2 => { self.dsp_reg_address = value; },
                0xf3 => { self.dsp.set_register(self.dsp_reg_address, value); },

                0xf4 ... 0xf9 => { self.ram[address as usize] = value; },

                0xfa => { self.timers[0].set_target(value); },
                0xfb => { self.timers[1].set_target(value); },
                0xfc => { self.timers[2].set_target(value); },

                _ => () // Do nothing
            }
        } else {
            self.ram[address as usize] = value;
        }
    }

    pub fn clear_echo_buffer(&mut self) {
        let length = self.dsp.calculate_echo_length();
        let mut end_addr = self.dsp.get_echo_start_address() as i32 + length;
        if end_addr > RAM_LEN as i32 {
            end_addr = RAM_LEN as i32;
        }
        for i in self.dsp.get_echo_start_address() as i32..end_addr {
            self.ram[i as usize] = 0xff;
        }
    }

    fn set_test_reg(&self, value: u8) {
        let _ = value;
        panic!("Test reg not yet implemented");
    }

    fn set_control_reg(&mut self, value: u8) {
        self.is_ipl_rom_enabled = (value & 0x80) != 0;
        if (value & 0x20) != 0 {
            self.write_u8(0xf6, 0x00);
            self.write_u8(0xf7, 0x00);
        }
        if (value & 0x10) != 0 {
            self.write_u8(0xf4, 0x00);
            self.write_u8(0xf5, 0x00);
        }
        self.timers[0].set_start_stop_bit((value & 0x01) != 0);
        self.timers[1].set_start_stop_bit((value & 0x02) != 0);
        self.timers[2].set_start_stop_bit((value & 0x04) != 0);
    }
}
