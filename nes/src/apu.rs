use bincode::{Decode, Encode};

use super::Nes;

static SAMPLE_FREQ: u32 = 40;

const PULSE_TABLE_SIZE: usize = 31;
const TND_TABLE_SIZE: usize = 203;

#[derive(Decode, Encode)]
pub struct Apu {
    /// APU cycle counter
    cycles: u16,

    pulse_1: Pulse<1>,
    pulse_2: Pulse<0>,
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,
    frame_counter: FrameCounter,

    // Lookup tables for the mixer
    pulse_table: [f32; PULSE_TABLE_SIZE],
    tnd_table: [f32; TND_TABLE_SIZE],

    sample_counter: u32,
    sample_sum: f32,
    pub samples: Vec<f32>,
}

impl Apu {
    pub(crate) fn new() -> Apu {
        let mut pulse_table = [0f32; 31];
        for n in 0..31 {
            pulse_table[n] = 95.52 / (8128f32 / n as f32 + 100f32);
        }

        let mut tnd_table = [0f32; 203];
        for n in 0..203 {
            tnd_table[n] = 163.67 / (24329f32 / n as f32 + 100f32);
        }

        Apu {
            cycles: 0,
            sample_counter: 0,
            sample_sum: 0.,

            pulse_1: Pulse::new(),
            pulse_2: Pulse::new(),
            triangle: Triangle::new(),
            noise: Noise::new(),
            dmc: Dmc::new(),
            frame_counter: FrameCounter::new(),

            pulse_table,
            tnd_table,

            samples: Vec::new(),
        }
    }

    fn quarter_frame_clock(&mut self) {
        self.pulse_1.envelope.clock();
        self.pulse_2.envelope.clock();
        self.triangle.clock_linear_counter();
        self.noise.envelope.clock();
    }

    fn half_frame_clock(&mut self) {
        self.pulse_1.half_frame_clock();
        self.pulse_2.half_frame_clock();
        self.triangle.length_counter.clock();
        self.noise.length_counter.clock();
    }

    // INVESTIGATE: implement using formula (https://www.nesdev.org/wiki/APU_Mixer) instead of lookup table
    // and compare performance and quality
    #[inline]
    fn mix_channels(&self) -> f32 {
        /*
        The APU mixer formulas can be efficiently implemented using two lookup tables: a 31-entry table
        for the two pulse channels and a 203-entry table for the remaining channels (due to the approximation
        of tnd_out, the numerators are adjusted slightly to preserve the normalized output range).

        output = pulse_out + tnd_out

        pulse_table [n] = 95.52 / (8128.0 / n + 100)

        pulse_out = pulse_table [pulse1 + pulse2]

        The tnd_out table is approximated (within 4%) by using a base unit close to the DMC's DAC.

        tnd_table [n] = 163.67 / (24329.0 / n + 100)

        tnd_out = tnd_table [3 * triangle + 2 * noise + dmc]
        */
        let pulse_1 = self.pulse_1.output() as usize;
        let pulse_2 = self.pulse_2.output() as usize;
        let pulse_out = self.pulse_table[pulse_1 + pulse_2];

        let triangle = self.triangle.output() as usize;
        let noise = self.noise.output() as usize;
        let dmc = self.dmc.output() as usize;
        let tnd_out = self.tnd_table[3 * triangle + 2 * noise + dmc];

        pulse_out + tnd_out
    }
}

impl Nes {
    #[inline]
    /// <https://wiki.nesdev.org/w/index.php?title=APU_Frame_Counter>
    pub(crate) fn apu_tick(&mut self) {
        // The channel timers are clocked on every CPU cycle, with the exception of
        // the pulse channels' timers which are clocked on every APU cycle.
        // Use CPU cycles so I can get "half-APU-cycle" timing correct...
        if self.apu.cycles % 2 == 0 {
            self.apu.pulse_1.clock();
            self.apu.pulse_2.clock();
        }

        self.apu.triangle.clock();
        self.apu.noise.clock();
        self.apu.dmc.clock();

        self.apu.cycles = self.apu.cycles.wrapping_add(1);

        if self.apu.frame_counter.mode {
            match self.apu.cycles {
                7457 => {
                    self.apu.quarter_frame_clock();
                }
                14913 => {
                    self.apu.quarter_frame_clock();
                    self.apu.half_frame_clock();
                }
                22371 => {
                    self.apu.quarter_frame_clock();
                }
                37281 => {
                    self.apu.quarter_frame_clock();
                    self.apu.half_frame_clock();
                }
                37282 => {
                    self.apu.cycles = 0;
                }
                _ => (),
            }
        } else {
            match self.apu.cycles {
                0 => {
                    //TODO: clock after writing to 4015
                }
                7457 => {
                    self.apu.quarter_frame_clock();
                }
                14913 => {
                    self.apu.quarter_frame_clock();
                    self.apu.half_frame_clock();
                }
                22371 => {
                    self.apu.quarter_frame_clock();
                }
                29828 => {
                    if !self.apu.frame_counter.irq_inhibit {
                        self.cpu.irq_signal = true;
                    }
                }
                29829 => {
                    self.apu.quarter_frame_clock();
                    self.apu.half_frame_clock();

                    if !self.apu.frame_counter.irq_inhibit {
                        self.cpu.irq_signal = true;
                    }
                }
                29830 => {
                    if !self.apu.frame_counter.irq_inhibit {
                        self.cpu.irq_signal = true;
                    }

                    self.apu.cycles = 0;
                }
                _ => (),
            }
        }

        // TODO: low / high pass filters, resampling to the target frequency
        self.apu.sample_counter += 1;
        let output = self.apu.mix_channels();
        self.apu.sample_sum += output;

        if self.apu.sample_counter == SAMPLE_FREQ {
            self.apu
                .samples
                .push(self.apu.sample_sum / SAMPLE_FREQ as f32);
            self.apu.sample_counter = 0;
            self.apu.sample_sum = 0.;
        }
    }

    /// <https://wiki.nesdev.org/w/index.php?title=APU_registers>
    #[inline]
    pub(crate) fn apu_write_reg(&mut self, addr: usize, val: u8) {
        match addr {
            0x4000 => self.apu.pulse_1.set_dlcv(val),
            0x4001 => self.apu.pulse_1.set_epns(val),
            0x4002 => self.apu.pulse_1.set_timer_low(val),
            0x4003 => self.apu.pulse_1.set_lh(val),
            0x4004 => self.apu.pulse_2.set_dlcv(val),
            0x4005 => self.apu.pulse_2.set_epns(val),
            0x4006 => self.apu.pulse_2.set_timer_low(val),
            0x4007 => self.apu.pulse_2.set_lh(val),
            0x4008 => self.apu.triangle.set_cr(val),
            0x400A => self.apu.triangle.set_timer_low(val),
            0x400B => self.apu.triangle.set_lh(val),
            0x400C => self.apu.noise.set_lcn(val),
            0x400E => self.apu.noise.set_lp(val),
            0x400F => self.apu.noise.set_length_counter(val),
            0x4010 => self.apu.dmc.set_ilf(val),
            0x4011 => self.apu.dmc.set_direct(val),
            0x4012 => self.apu.dmc.set_sample_address(val),
            0x4013 => self.apu.dmc.set_sample_length(val),
            0x4015 => self.apu_write_status(val),
            0x4017 => {
                /*
                Writing to $4017 with bit 7 set ($80) will immediately clock all of its controlled units
                at the beginning of the 5-step sequence; with bit 7 clear, only the sequence is reset
                without clocking any of its units.
                */
                if val & 0x80 != 0 {
                    self.apu.quarter_frame_clock();
                    self.apu.half_frame_clock();
                }

                self.apu.cycles = 0;
                self.apu.frame_counter.set_mi(val, &mut self.cpu.irq_signal)
            }
            _ => (),
        }
    }

    /// <https://wiki.nesdev.org/w/index.php?title=APU#Status_.28.244015.29>
    #[inline]
    pub(crate) fn apu_read_status(&mut self) -> u8 {
        let mut result = 0;
        if self.apu.pulse_1.length_counter.counter > 0 {
            result |= 1;
        }

        if self.apu.pulse_2.length_counter.counter > 0 {
            result |= 2;
        }

        if self.apu.triangle.length_counter.counter > 0 {
            result |= 4;
        }

        if self.apu.noise.length_counter.counter > 0 {
            result |= 8;
        }

        //TODO: set DMC active bit

        if self.apu.frame_counter.irq_inhibit {
            result |= 0x40;
        }

        if self.apu.dmc.irq_enable {
            result |= 0x80;
        }

        self.apu.frame_counter.irq_inhibit = false;
        self.cpu.irq_signal = false;

        result
    }

    /*
    $4015 write ---D NT21   Enable DMC (D), noise (N), triangle (T), and pulse channels (2/1)
    Writing a zero to any of the channel enable bits will silence that channel and immediately set its length counter to 0.
    If the DMC bit is clear, the DMC bytes remaining will be set to 0 and the DMC will silence when it empties.
    If the DMC bit is set, the DMC sample will be restarted only if its bytes remaining is 0. If there are bits remaining in the 1-byte sample buffer, these will finish playing before the next sample is fetched.
    Writing to this register clears the DMC interrupt flag.
    */
    #[inline]
    fn apu_write_status(&mut self, val: u8) {
        self.apu.dmc.irq_enable = false;

        let _d = val & 0x10 != 0;
        let n = val & 8 != 0;
        let t = val & 4 != 0;
        let p_2 = val & 2 != 0;
        let p_1 = val & 1 != 0;

        //TODO: manage DMC

        if !n {
            self.apu.noise.length_counter.counter = 0;
        }
        self.apu.noise.length_counter.enabled = n;

        if !t {
            self.apu.triangle.length_counter.counter = 0;
        }
        self.apu.triangle.length_counter.enabled = t;

        if !p_2 {
            self.apu.pulse_2.length_counter.counter = 0;
        }
        self.apu.pulse_2.length_counter.enabled = p_2;

        if !p_1 {
            self.apu.pulse_1.length_counter.counter = 0;
        }
        self.apu.pulse_1.length_counter.enabled = p_1;
    }
}

/// <https://wiki.nesdev.org/w/index.php?title=APU_Frame_Counter>
#[derive(Decode, Encode)]
struct FrameCounter {
    /// true - 5-step mode, false -4-step mode
    mode: bool,
    /// Counts whether the current cycle is an odd or even CPU cycle
    odd_cycle: bool,
    irq_inhibit: bool,
}

impl FrameCounter {
    fn new() -> FrameCounter {
        FrameCounter {
            mode: false,
            odd_cycle: false,
            irq_inhibit: true,
        }
    }

    /*
    $4017   MI--.----   Set mode and interrupt (write)
    Bit 7   M--- ----   Sequencer mode: 0 selects 4-step sequence, 1 selects 5-step sequence
    Bit 6   -I-- ----   Interrupt inhibit flag. If set, the frame interrupt flag is cleared,
    otherwise it is unaffected.

    Side effects: After 3 or 4 CPU clock cycles*, the timer is reset.
    If the mode flag is set, then both "quarter frame" and "half frame" signals are also generated
    */
    #[inline]
    fn set_mi(&mut self, val: u8, irq_signal: &mut bool) {
        self.mode = val & 0x80 != 0;
        self.irq_inhibit = val & 0x40 != 0;

        if self.irq_inhibit {
            *irq_signal = false;
        }
    }
}

/// <https://wiki.nesdev.org/w/index.php?title=APU_Pulse>
#[derive(Decode, Encode)]
struct Pulse<const ADDER: u16> {
    /// At which step of the 8-step sequence it currently is
    duty_cycle: u8,
    /// Which of the 4 8-step duty cycle sequences is currently selected
    duty_seq: u8,

    envelope: Envelope,
    sweep: Sweep<ADDER>,
    length_counter: LengthCounter,
}

impl<const ADDER: u16> Pulse<ADDER> {
    fn new() -> Pulse<ADDER> {
        Pulse {
            duty_cycle: 0,
            duty_seq: 0,
            envelope: Envelope::new(),

            sweep: Sweep::new(),
            length_counter: LengthCounter::new(),
        }
    }

    /*
    $4000 DDlc.vvvv Pulse 1 Duty cycle, length counter halt, constant volume/envelope flag,
    and volume/envelope divider period
    */
    #[inline]
    fn set_dlcv(&mut self, val: u8) {
        self.duty_seq = (val & 0xC0) >> 3;
        self.length_counter.halt = (val & 0x20) != 0;
        self.envelope._loop = (val & 0x20) != 0;
        self.envelope.constant_volume = (val & 0x10) != 0;
        self.envelope.period_reload = val & 0xF;
    }

    #[inline]
    fn set_epns(&mut self, val: u8) {
        self.sweep.set_epns(val);
    }

    /*
    $4002 LLLL.LLLL    Pulse 1 timer Low 8 bits
    */
    #[inline]
    fn set_timer_low(&mut self, val: u8) {
        self.sweep.timer_reload = (self.sweep.timer_reload & !0xFF) | u16::from(val);
    }

    /*
    $4003 llll.lHHH    Pulse 1 length counter load and timer High 3 bits

    Side effects - The sequencer is immediately restarted at the first value of the current sequence.
    The envelope is also restarted. The period divider is not reset.
    */
    #[inline]
    fn set_lh(&mut self, val: u8) {
        self.envelope.start = true;
        self.duty_cycle = 0;
        self.length_counter.set_counter((val & 0xF8) >> 3);
        self.sweep.timer_reload = (self.sweep.timer_reload & !0x700) | (u16::from(val & 7) << 8);
    }

    #[inline]
    fn clock(&mut self) {
        if self.sweep.timer > 0 {
            self.sweep.timer -= 1;
        } else {
            /*
            The reason for the odd output from the sequencer is that the counter is initialized to zero
            but counts downward rather than upward. Thus it reads the sequence lookup table in the
            order 0, 7, 6, 5, 4, 3, 2, 1.
            */
            if self.duty_cycle == 0 {
                self.duty_cycle = 7;
            } else {
                self.duty_cycle -= 1;
            }

            self.sweep.timer = self.sweep.timer_reload;
        }
    }

    #[inline]
    fn half_frame_clock(&mut self) {
        self.length_counter.clock();
        self.sweep.clock();
    }

    /*
    Duty  Sequence lookup table   Output waveform
    0     0 0 0 0 0 0 0 1         0 1 0 0 0 0 0 0 (12.5%)
    1     0 0 0 0 0 0 1 1         0 1 1 0 0 0 0 0 (25%)
    2     0 0 0 0 1 1 1 1         0 1 1 1 1 0 0 0 (50%)
    3     1 1 1 1 1 1 0 0         1 0 0 1 1 1 1 1 (25% negated)
    */
    #[rustfmt::skip]
    const DUTY_SEQUENCE: [bool; 0x20] = [
        false, false, false, false, false, false, false, true,
        false, false, false, false, false, false, true,  true,
        false, false, false, false, true,  true,  true,  true,
        true,  true,  true,  true,  true,  true,  false, false,
    ];

    #[inline]
    fn output(&self) -> u8 {
        let active = Self::DUTY_SEQUENCE[(self.duty_seq | self.duty_cycle) as usize];
        /*
        The mixer receives the current envelope volume except when The sequencer output is zero,
        or overflow from the sweep unit's adder is silencing the channel, or the length counter is
        zero, or the timer has a value less than eight.
        */
        if active
            && self.length_counter.counter > 0
            && self.sweep.timer >= 8
            && self.sweep.period < 0x800
        {
            self.envelope.get_volume()
        } else {
            0
        }
    }
}

#[derive(Decode, Encode)]
struct Triangle {
    /// Linear counter reload value set by registers, reloaded when timer reaches 0
    linear_counter_reload: u8,
    /// Current linear counter value
    linear_counter: u8,
    /// Whether the linear counter should be reloaded on the next quarter frame clock
    linear_counter_reload_flag: bool,
    /// Whether the reload flag should not be reset on the next quarter frame clock
    linear_counter_control: bool,

    /// Current timer value
    timer: u16,
    /// Timer reload value set by registerrs
    timer_reload: u16,
    /// Current step in the output sequence
    sequence_step: u8,

    length_counter: LengthCounter,
}

impl Triangle {
    fn new() -> Triangle {
        Triangle {
            linear_counter_control: false,
            linear_counter_reload: 0,
            linear_counter_reload_flag: false,
            linear_counter: 0,

            timer: 0,
            timer_reload: 0,
            sequence_step: 0,

            length_counter: LengthCounter::new(),
        }
    }

    /*
    $4008   CRRR.RRRR   Linear counter setup (write)
    bit 7   C---.----   Control flag (this bit is also the length counter halt flag)
    bits 6-0-RRR RRRR   Counter reload value
    */
    #[inline]
    fn set_cr(&mut self, val: u8) {
        self.linear_counter_control = val & 0x80 != 0;
        self.length_counter.halt = val & 0x80 != 0;
        self.linear_counter_reload = val & 0x7F;
    }

    /*
    $400A   LLLL.LLLL   Timer low (write)
    bits 7-0LLLL LLLL   Timer low 8 bits
    */
    #[inline]
    fn set_timer_low(&mut self, val: u8) {
        self.timer_reload = (self.timer_reload & !0xFF) | u16::from(val);
    }

    /*
    $400B   llll.lHHH   Length counter load and timer high (write)
    bits 2-0---- -HHH   Timer high 3 bits
    Side effects: Sets the linear counter reload flag
    */
    #[inline]
    fn set_lh(&mut self, val: u8) {
        self.length_counter.set_counter((val & 0xF8) >> 3);
        self.timer_reload = (self.timer_reload & !0x700) | (u16::from(val & 7) << 8);
        self.linear_counter_reload_flag = true;
    }

    #[inline]
    fn clock_linear_counter(&mut self) {
        /*
        When the frame counter generates a linear counter clock, the following actions occur in order:

        1. If the linear counter reload flag is set, the linear counter is reloaded with the counter reload
        value, otherwise if the linear counter is non-zero, it is decremented.

        2. If the control flag is clear, the linear counter reload flag is cleared.
        */
        if self.linear_counter_reload_flag {
            self.linear_counter = self.linear_counter_reload;
        } else if self.linear_counter != 0 {
            self.linear_counter -= 1;
        }

        if !self.linear_counter_control {
            self.linear_counter_reload_flag = false;
        }
    }

    #[inline]
    fn clock(&mut self) {
        if self.length_counter.counter != 0 && self.linear_counter != 0 {
            if self.timer > 0 {
                self.timer -= 1;
            } else {
                self.sequence_step += 1;
                if self.sequence_step == 32 {
                    self.sequence_step = 0;
                }

                self.timer = self.timer_reload;
            }
        }
    }

    #[rustfmt::skip]
    const SEQUENCE: [u8; 32] = [
        15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0,
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
    ];

    #[inline]
    fn output(&self) -> u8 {
        /*
        https://www.nesdev.org/wiki/APU_Triangle

        Unlike the pulse channels, the triangle channel supports frequencies up to the
        maximum frequency the timer will allow, meaning frequencies up to fCPU/32
        (about 55.9 kHz for NTSC) are possible - far above the audible range.
        Some games, e.g. Mega Man 2, "silence" the triangle channel by setting the timer
        to zero, which produces a popping sound when an audible frequency is resumed,
        easily heard e.g. in Crash Man's stage.
        */

        /*
        Write a period value of 0 or 1 to $400A/$400B, causing a very high frequency.
        Due to the averaging effect of the lowpass filter, the resulting value is halfway between 7 and 8.
        This sudden jump to "7.5" causes a harder popping noise than other triangle silencing methods,
        which will instead halt it in whatever its current output position is.
        Mega Man 1 and 2 use this technique.
        */
        if self.timer_reload < 2 {
            return 7;
        }

        /*
        Silencing the triangle channel merely halts it.
        It will continue to output its last value, rather than 0.
        */
        Self::SEQUENCE[self.sequence_step as usize]
    }
}

#[derive(Decode, Encode)]
struct Noise {
    /// Timer reload value set by registers, reloaded when timer reaches 0
    timer_period_reload: u16,
    /// Current timer value
    timer: u16,

    /// Which bit to use when calculating the LSFR feedback (true - bit 6, false - bit 1)
    shift_feedback_mode: bool,
    shift_reg: u16,

    envelope: Envelope,
    length_counter: LengthCounter,
}

impl Noise {
    fn new() -> Noise {
        Noise {
            shift_feedback_mode: false,
            timer_period_reload: 0,
            timer: 0,

            shift_reg: 1,

            envelope: Envelope::new(),
            length_counter: LengthCounter::new(),
        }
    }

    /*
    $400C --LC NNNN Length counter halt, constant volume/envelope flag,
    and volume/envelope divider period (write)
    */
    #[inline]
    fn set_lcn(&mut self, val: u8) {
        self.length_counter.halt = (val & 0x20) != 0;
        self.envelope.constant_volume = (val & 0x10) != 0;
        self.envelope.period_reload = val & 0xF;
    }

    const TIMER_PERIODS: [u16; 0x10] = [
        4, 8, 16, 32, 64, 96, 128, 160, 202, 254, 380, 508, 762, 1016, 2034, 4068,
    ];

    /*
    $400E L--- PPPP   Mode flag, noise period
    */
    #[inline]
    fn set_lp(&mut self, val: u8) {
        self.shift_feedback_mode = (val & 0x80) != 0;
        // INVESTIGATE: according to MESEN this should be minus one ?
        self.timer_period_reload = Self::TIMER_PERIODS[(val & 0xF) as usize] - 1;
    }

    /*
    $400F LLLL L---   Length counter load (also restarts envelope)
    */
    #[inline]
    fn set_length_counter(&mut self, val: u8) {
        self.envelope.start = true;
        self.length_counter.set_counter((val & 0xF8) >> 3);
    }

    #[inline]
    fn clock(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period_reload;
            /*
            When the timer clocks the shift register, the following actions occur in order:

            Feedback is calculated as the exclusive-OR of bit 0 and one other bit: bit 6 if Mode flag is set,
            otherwise bit 1. The shift register is shifted right by one bit.
            Bit 14, the leftmost bit, is set to the feedback calculated earlier.
            */
            let feedback = if self.shift_feedback_mode {
                (self.shift_reg & 1) ^ ((self.shift_reg >> 6) & 1)
            } else {
                (self.shift_reg & 1) ^ ((self.shift_reg >> 1) & 1)
            };

            self.shift_reg >>= 1;
            self.shift_reg |= feedback << 14;
        } else {
            self.timer -= 1;
        }
    }

    #[inline]
    fn output(&self) -> u8 {
        /*
        The mixer receives the current envelope volume except when

        Bit 0 of the shift register is set, or
        The length counter is zero
        */
        if ((self.shift_reg & 1) == 0) && self.length_counter.counter != 0 {
            self.envelope.get_volume()
        } else {
            0
        }
    }
}

#[derive(Decode, Encode)]
struct Dmc {
    irq_enable: bool,
    /// Id the DPCM sample playback should be looped
    loop_enable: bool,
    playback_rate: u16,

    /// Address of the DPCM sample
    sample_address: u16,
    /// DPCM sample length in bytes
    sample_length: u16,

    /// Holds the current 8 DPCM samples
    sample_buffer: u8,

    output_level: u8,
}

impl Dmc {
    fn new() -> Dmc {
        Dmc {
            irq_enable: false,
            loop_enable: false,
            playback_rate: 0,

            sample_address: 0,
            sample_length: 0,

            sample_buffer: 0,

            output_level: 0,
        }
    }

    const FREQ_TABLE: [u16; 16] = [
        428, 380, 340, 320, 286, 254, 226, 214, 190, 160, 142, 128, 106, 84, 72, 54,
    ];

    /*
    $4010 IL-- FFFF    IRQ enable, loop sample, frequency index
    */
    #[inline]
    fn set_ilf(&mut self, val: u8) {
        self.irq_enable = (val & 0x80) != 0;
        self.loop_enable = (val & 0x40) != 0;
        self.playback_rate = Self::FREQ_TABLE[(val & 0xF) as usize];
    }

    /*
    $4011 -DDD DDDD    Direct load
    The DMC output level is set to D, an unsigned value.
    */
    #[inline]
    fn set_direct(&mut self, val: u8) {
        self.output_level = val & 0x7F;
    }

    /*
    $4012 AAAA AAAA    Sample address = %11AAAAAA.AA000000 = $C000 + (A * 64)
    */
    #[inline]
    fn set_sample_address(&mut self, val: u8) {
        self.sample_address = 0xC000 | (u16::from(val) << 6);
    }

    /*
    $4013 LLLL LLLL    Sample length = %LLLL.LLLL0001 = (L * 16) + 1 bytes
    */
    #[inline]
    fn set_sample_length(&mut self, val: u8) {
        self.sample_length = 1 | (u16::from(val) << 4);
    }

    #[inline]
    fn clock(&mut self) {}

    #[inline]
    fn output(&self) -> u8 {
        self.output_level
    }
}

#[derive(Decode, Encode)]
struct LengthCounter {
    /// If the channel connected to this length counter is enabled
    enabled: bool,
    /// If the length counter is halted
    halt: bool,
    counter: u8,
}

impl LengthCounter {
    fn new() -> LengthCounter {
        LengthCounter {
            enabled: true,
            halt: false,
            counter: 0,
        }
    }

    const LENGTH_TABLE: [u8; 0x20] = [
        10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96,
        22, 192, 24, 72, 26, 16, 28, 32, 30,
    ];

    #[inline]
    fn set_counter(&mut self, val: u8) {
        if self.enabled {
            self.counter = Self::LENGTH_TABLE[val as usize];
        }
    }

    #[inline]
    fn clock(&mut self) {
        if self.enabled {
            if self.counter > 0 && !self.halt {
                self.counter -= 1;
            }
        } else {
            self.counter = 0;
        }
    }
}

#[derive(Decode, Encode)]
struct Sweep<const ADDER: u16> {
    /// Whether the sweep unit is enabled
    enabled: bool,
    /// The number of bits the timer_reload is shifted to the right
    shift: u8,
    /// If the change calculated from the current timer_reload and shift should
    /// be made negative
    negate: bool,

    /// Divider reload value set by registers, reloaded on the next quarter frame clock
    /// when counter == 0 or when the reload flag is true
    period: u16,
    /// Current divider value
    counter: u16,
    /// If the counter should be reloaded on the next quarter frame clock
    reload: bool,

    /// The pulse chanel timer value
    timer: u16,
    /// The timer reload value set bys register, reloaded when timer reaches 0
    timer_reload: u16,
}

impl<const ADDER: u16> Sweep<ADDER> {
    fn new() -> Sweep<ADDER> {
        Sweep {
            enabled: false,
            negate: false,
            shift: 0,

            period: 0,
            counter: 0,
            reload: false,

            timer: 0,
            timer_reload: 0,
        }
    }

    /*
    $4005 EPPP.NSSS Pulse channel 2 sweep setup (write)
    7      E--- ---- Enabled flag
    6-4    -PPP ---- The divider's period is P + 1 half-frames
    3      ---- N--- Negate flag
        0: add to period, sweeping toward lower frequencies
        1: subtract from period, sweeping toward higher frequencies

    2-0    ---- -SSS Shift count (number of bits)

    Side effect -  Sets the reload flag
    */
    #[inline]
    fn set_epns(&mut self, val: u8) {
        self.enabled = (val & 0x80) != 0;
        self.period = (val as u16 & 0x70) >> 4;
        self.negate = (val & 8) != 0;
        self.shift = val & 7;
        self.reload = true;
    }

    #[inline]
    fn clock(&mut self) {
        /*
        The sweep unit continuously calculates each channel's target period in this way:

        A barrel shifter shifts the channel's 11-bit raw timer period right by the shift count,
        producing the change amount. If the negate flag is true, the change amount is made negative.
        The target period is the sum of the current period and the change amount.

        For example, if the negate flag is false and the shift amount is zero, the change amount
        equals the current period, making the target period equal to twice the current period.

        The two pulse channels have their adders' carry inputs wired differently, which produces
        different results when each channel's change amount is made negative:

        Pulse 1 adds the ones' complement (−c − 1). Making 20 negative produces a change amount
        of −21.
        Pulse 2 adds the two's complement (−c). Making 20 negative produces a change amount of −20.
        */
        let target_period = {
            let change = self.timer_reload >> self.shift;
            if !self.negate {
                self.timer_reload.wrapping_add(change)
            } else {
                self.timer_reload.wrapping_sub(change + ADDER)
            }
        };

        /*
        When the frame counter sends a half-frame clock (at 120 or 96 Hz), two things happen.
        If the divider's counter is zero, the sweep is enabled, and the sweep unit is not muting the
        channel: The pulse's period is adjusted.

        If the divider's counter is zero or the reload flag is true: The counter is set to P and the
        reload flag is cleared. Otherwise, the counter is decremented.

        Two conditions cause the sweep unit to mute the channel:

        If the current period is less than 8, the sweep unit mutes the channel.
        If at any time the target period is greater than $7FF, the sweep unit mutes the channel.
        */
        if self.enabled && self.counter == 0 && self.timer_reload >= 8 && target_period <= 0x7FF {
            self.timer_reload = target_period;
        }

        if self.counter == 0 || self.reload {
            self.counter = self.period + 1;
            self.reload = false;
        } else {
            self.counter -= 1;
        }
    }
}

#[derive(Decode, Encode)]
struct Envelope {
    start: bool,
    period_reload: u8,
    period: u8,
    constant_volume: bool,
    decay_counter: u8,
    _loop: bool,
}

impl Envelope {
    fn new() -> Envelope {
        Envelope {
            start: false,
            period_reload: 0,
            period: 0,
            constant_volume: false,
            decay_counter: 0,
            _loop: false,
        }
    }

    /*
    When clocked by the frame counter, one of two actions occurs: if the start flag is clear, the divider
    is clocked, otherwise the start flag is cleared, the decay level counter is loaded with 15, and the
    divider's period is immediately reloaded.

    When the divider is clocked while at 0, it is loaded with V and clocks the decay level counter.
    Then one of two actions occurs: If the counter is non-zero, it is decremented, otherwise if the
    loop flag is set, the decay level counter is loaded with 15.
    */
    #[inline]
    fn clock(&mut self) {
        if !self.start {
            if self.period == 0 {
                self.period = self.period_reload;
                if self.decay_counter != 0 {
                    self.decay_counter -= 1;
                } else if self._loop {
                    self.decay_counter = 15;
                }
            } else {
                self.period -= 1;
            }
        } else {
            self.start = false;
            self.decay_counter = 15;
            self.period = self.period_reload;
        }
    }

    #[inline]
    fn get_volume(&self) -> u8 {
        /*
        The envelope unit's volume output depends on the constant volume flag: if set, the
        envelope parameter directly sets the volume, otherwise the decay level is the current
        volume. The constant volume flag has no effect besides selecting the volume source;
        the decay level will still be updated when constant volume is selected.
        */
        if self.constant_volume {
            self.period_reload
        } else {
            self.decay_counter
        }
    }
}

/* pub struct ApuChannelsOut {
    pulse_1: u8,
    pulse_2: u8,
}
 */
