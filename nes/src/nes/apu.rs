use super::Nes;

pub struct Apu {
    cycles: u16,
    pulse_1: Pulse,
    pulse_2: Pulse,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,
    frame_counter: FrameCounter,

    cycle: bool,

    pub nes: *mut Nes,
}

impl Apu {
    pub fn new() -> Apu {
        Apu {
            cycles: 0,
            pulse_1: Pulse::new(),
            pulse_2: Pulse::new(),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),
            frame_counter: FrameCounter::new(),

            cycle: false,

            nes: 0 as *mut Nes,
        }
    }

    #[inline]
    pub fn tick(&mut self) {
        if self.cycle {
            self.cycles += 1;

            if self.frame_counter.mode {
                match self.cycles {
                    //TODO: clock envelopes and sweeps
                    3728 => {}
                    7456 => {
                        self.pulse_1.length_counter.clock();
                        self.pulse_2.length_counter.clock();
                        self.noise.length_counter.clock();
                    }
                    11185 => {}
                    18640 => {
                        self.pulse_1.length_counter.clock();
                        self.pulse_2.length_counter.clock();
                        self.noise.length_counter.clock();
                        self.cycles = 0
                    }
                    _ => (),
                }
            } else {
                match self.cycles {
                    //TODO: clock envelopes and sweeps
                    3728 => {}
                    7456 => {
                        self.pulse_1.length_counter.clock();
                        self.pulse_2.length_counter.clock();
                        self.noise.length_counter.clock();
                    }
                    11185 => {}
                    14914 => {
                        self.pulse_1.length_counter.clock();
                        self.pulse_2.length_counter.clock();
                        self.noise.length_counter.clock();
                        self.cycles = 0
                    }
                    _ => (),
                }
            }
        }

        self.cycle = !self.cycle;
    }

    #[inline]
    pub fn write_reg(&mut self, addr: usize, val: u8) {
        match addr {
            0x4000 => self.pulse_1.set_dlcv(val),
            0x4001 => self.pulse_1.set_epns(val),
            0x4002 => self.pulse_1.set_t(val),
            0x4003 => self.pulse_1.set_lt(val),
            0x4004 => self.pulse_2.set_dlcv(val),
            0x4005 => self.pulse_2.set_epns(val),
            0x4006 => self.pulse_2.set_t(val),
            0x4007 => self.pulse_2.set_lt(val),
            0x4008 => self.triangle.set_c(val),
            0x400A => self.triangle.set_tl(val),
            0x400B => self.triangle.set_l(val),
            0x400C => self.noise.set_lcn(val),
            0x400E => self.noise.set_lp(val),
            0x400F => self.noise.set_l(val),
            0x4010 => self.dmc.set_ilf(val),
            0x4011 => self.dmc.set_d(val),
            0x4012 => self.dmc.set_a(val),
            0x4013 => self.dmc.set_l(val),
            0x4015 => self.write_status(val),
            0x4017 => self.frame_counter.set_mi(val),
            _ => (),
        }
    }

    //$4015 read  IF-D NT21   DMC interrupt (I), frame interrupt (F), DMC active (D), length counter > 0 (N/T/2/1)
    //N/T/2/1 will read as 1 if the corresponding length counter is greater than 0. For the triangle channel, the status of the linear counter is irrelevant.
    //D will read as 1 if the DMC bytes remaining is more than 0.
    //Reading this register clears the frame interrupt flag (but not the DMC interrupt flag).
    //If an interrupt flag was set at the same moment of the read, it will read back as 1 but it will not be cleared.
    #[inline]
    pub fn read_status(&mut self) -> u8 {
        let mut result = 0;
        if self.pulse_1.length_counter.counter > 0 {
            result |= 1;
        }

        if self.pulse_2.length_counter.counter > 0 {
            result |= 2;
        }

        //if self.triangle.length_counter.counter > 0 {
        //    result |= 4;
        //}

        if self.noise.length_counter.counter > 0 {
            result |= 8;
        }

        //TODO: set DMC active bit

        if self.frame_counter.irq_inhibit {
            result |= 0x40;
        }

        if self.dmc.irq_enable {
            result |= 0x80;
        }

        self.frame_counter.irq_inhibit = false;

        result
    }

    //$4015 write ---D NT21   Enable DMC (D), noise (N), triangle (T), and pulse channels (2/1)
    //Writing a zero to any of the channel enable bits will silence that channel and immediately set its length counter to 0.
    //If the DMC bit is clear, the DMC bytes remaining will be set to 0 and the DMC will silence when it empties.
    //If the DMC bit is set, the DMC sample will be restarted only if its bytes remaining is 0. If there are bits remaining in the 1-byte sample buffer, these will finish playing before the next sample is fetched.
    //Writing to this register clears the DMC interrupt flag.
    #[inline]
    fn write_status(&mut self, val: u8) {
        self.dmc.irq_enable = false;

        let d = val & 0x10 != 0;
        let n = val & 8 != 0;
        let t = val & 4 != 0;
        let p_2 = val & 2 != 0;
        let p_1 = val & 1 != 0;

        //TODO: manage DMC

        if !n {
            self.noise.volume = 0;
            self.noise.length_counter.counter = 0;
        }

        //if !t {
        //    self.triangle.length_counter.counter = 0;
        //}

        if !p_2 {
            self.pulse_2.volume = 0;
            self.pulse_2.length_counter.counter = 0;
        }

        if !p_1 {
            self.pulse_1.volume = 0;
            self.pulse_1.length_counter.counter = 0;
        }
    }
}

//The pulse channels produce a variable-width pulse signal, controlled by volume, envelope, length, and sweep units.
//$4000 / $4004   DDLC VVVV   Duty (D), envelope loop / length counter halt (L), constant volume (C), volume/envelope (V)
//  Side effects: The duty cycle is changed (see table below), but the sequencer's current position isn't affected.
//$4001 / $4005   EPPP NSSS   Sweep unit: enabled (E), period (P), negate (N), shift (S)
//$4002 / $4006   TTTT TTTT   Timer low (T)
//$4003 / $4007   LLLL LTTT   Length counter load (L), timer high (T)
//  Side effects: The sequencer is immediately restarted at the first value of the current sequence. The envelope is also restarted. The period divider is not reset.[1]
struct Pulse {
    duty: u8,
    constant_volume: bool,
    volume: u8,

    timer: u16,

    sweep_unit: SweepUnit,
    length_counter: LengthCounter,
}

impl Pulse {
    pub fn new() -> Pulse {
        Pulse {
            duty: 0,
            constant_volume: false,
            volume: 0,

            timer: 0,

            sweep_unit: SweepUnit::new(),
            length_counter: LengthCounter::new(),
        }
    }

    #[inline]
    pub fn set_dlcv(&mut self, val: u8) {
        //TODO:change duty cycle
        self.duty = (val & 0xC0) >> 6;
        self.length_counter.halt = (val & 0x20) != 0;
        self.constant_volume = (val & 0x10) != 0;
        self.volume = val & 0xF;
    }

    #[inline]
    pub fn set_epns(&mut self, val: u8) {
        self.sweep_unit.load(val);
    }

    #[inline]
    pub fn set_t(&mut self, val: u8) {
        self.timer = (self.timer & !0xFF) | u16::from(val);
    }

    #[inline]
    pub fn set_lt(&mut self, val: u8) {
        //TODO:restart sequencer
        self.length_counter.load((val & 0xF8) >> 3);
        self.timer = (self.timer & !0x700) | (u16::from(val & 7) << 8);
    }
}

//$4008   CRRR.RRRR   Linear counter setup (write)
//bit 7   C---.----   Control flag (this bit is also the length counter halt flag)
//bits 6-0-RRR RRRR   Counter reload value
//
//$400A   LLLL.LLLL   Timer low (write)
//bits 7-0LLLL LLLL   Timer low 8 bits
//
//$400B   llll.lHHH   Length counter load and timer high (write)
//bits 2-0---- -HHH   Timer high 3 bits
//Side effects: Sets the linear counter reload flag
struct Triangle {
    counter_control: bool,
    counter_reload: u8,
    timer: u16,

    length_counter_load: u8,
}

impl Triangle {
    pub fn new() -> Triangle {
        Triangle {
            counter_control: false,
            counter_reload: 0,
            timer: 0,

            length_counter_load: 0,
        }
    }

    #[inline]
    pub fn set_c(&mut self, val: u8) {
        self.counter_control = val & 0x80 != 0;
        self.counter_reload = val & 0x7F;
    }

    #[inline]
    pub fn set_tl(&mut self, val: u8) {
        self.timer = (self.timer & !0xFF) | u16::from(val);
    }

    #[inline]
    pub fn set_l(&mut self, val: u8) {
        self.length_counter_load = (val & 0xF8) >> 3;
        self.timer = (self.timer & !0x700) | (u16::from(val & 7) << 8);
        //TODO: set linear control reload flag
    }
}

static PERIOD_NOISE: [u16; 0x10] = [
    4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
];

//$400C   --LC NNNN   Loop envelope/disable length counter, constant volume, envelope period/volume
//$400E   L--- PPPP   Loop noise, noise period
//$400F   LLLL L---   Length counter load (also starts envelope)
struct Noise {
    constant_volume: bool,
    volume: u8,

    loop_noise: bool,
    noise_period: u8,

    length_counter: LengthCounter,
}

impl Noise {
    pub fn new() -> Noise {
        Noise {
            constant_volume: false,
            volume: 0,

            loop_noise: false,
            noise_period: 0,

            length_counter: LengthCounter::new(),
        }
    }

    #[inline]
    pub fn set_lcn(&mut self, val: u8) {
        self.length_counter.halt = (val & 0x20) != 0;
        self.constant_volume = (val & 0x10) != 0;
        self.volume = val & 0xF;
    }

    #[inline]
    pub fn set_lp(&mut self, val: u8) {
        self.loop_noise = (val & 0x80) != 0;
        self.noise_period = val & 0xF;
    }

    #[inline]
    pub fn set_l(&mut self, val: u8) {
        self.length_counter.load((val & 0xF8) >> 3);
    }
}

//$4010   IL-- FFFF   IRQ enable, loop sample, frequency index
//$4011   -DDD DDDD   Direct load
//$4012   AAAA AAAA   Sample address %11AAAAAA.AA000000
//$4013   LLLL LLLL   Sample length %0000LLLL.LLLL0001
struct Dmc {
    irq_enable: bool,
    loop_sample: bool,
    frequency_index: u8,

    direct_load: u8,

    sample_address: u16,

    sample_length: u16,
}

impl Dmc {
    pub fn new() -> Dmc {
        Dmc {
            irq_enable: false,
            loop_sample: false,
            frequency_index: 0,

            direct_load: 0,
            sample_address: 0,

            sample_length: 0,
        }
    }

    #[inline]
    pub fn set_ilf(&mut self, val: u8) {
        self.irq_enable = (val & 0x80) != 0;
        self.loop_sample = (val & 0x40) != 0;
        self.frequency_index = val & 0xF;
    }

    #[inline]
    pub fn set_d(&mut self, val: u8) {
        self.direct_load = val & 0x7F;
    }

    #[inline]
    pub fn set_a(&mut self, val: u8) {
        self.sample_address = 0xC000 | (u16::from(val) << 6);
    }

    #[inline]
    pub fn set_l(&mut self, val: u8) {
        self.sample_length = 1 | (u16::from(val) << 4);
    }
}

struct FrameCounter {
    mode: bool, //true -5-step, false-4-step
    irq_inhibit: bool,
}

impl FrameCounter {
    pub fn new() -> FrameCounter {
        FrameCounter {
            mode: false,
            irq_inhibit: true,
        }
    }

    #[inline]
    pub fn set_mi(&mut self, val: u8) {
        self.mode = val & 0x80 != 0;
        self.irq_inhibit = val & 0x40 != 0;
    }
}

static LENGTH_TABLE: [u8; 0x20] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

struct LengthCounter {
    halt: bool,
    counter: u8,
}

impl LengthCounter {
    pub fn new() -> LengthCounter {
        LengthCounter {
            halt: false,
            counter: 0,
        }
    }

    #[inline]
    pub fn load(&mut self, val: u8) {
        self.counter = LENGTH_TABLE[val as usize];
    }

    #[inline]
    pub fn clock(&mut self) {
        if self.counter > 0 && !self.halt {
            self.counter -= 1;
        }
    }
}

struct SweepUnit {
    enabled: bool,
    negate: bool,
    shift: u8,

    period: u8,
    counter: u8,

    reload: bool,
}

impl SweepUnit {
    pub fn new() -> SweepUnit {
        SweepUnit {
            enabled: false,
            negate: false,
            shift: 0,

            period: 0,
            counter: 0,

            reload: false,
        }
    }

    #[inline]
    pub fn load(&mut self, val: u8) {
        self.enabled = (val & 0x80) != 0;
        self.period = (val & 0x70) >> 4;
        self.negate = (val & 8) != 0;
        self.shift = val & 7;
        self.reload = true;
    }

    #[inline]
    pub fn clock(&mut self) {
        let mute = false;
        if self.counter == 0 && self.enabled && !mute {
            //adjust pulse period
        }

        if self.counter == 0 || self.reload {
            self.counter = self.period;
            self.reload = false;
        } else {
            self.counter = self.counter.wrapping_sub(1);
        }
    }
}
