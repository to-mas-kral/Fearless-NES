/*
    Rust rewrite of:
    blip_buf 1.1.0. http://www.slack.net/~ant/ by Shay Green.
*/

use bincode::{Decode, Encode};

#[derive(Decode, Encode)]
pub struct BlipBuf<const S: usize> {
    factor: u64,
    offset: u64,
    available: u64,
    integrator: i32,

    buf: [i32; S],
    last_sample: i32,
    time: u64,
}

impl<const S: usize> BlipBuf<S> {
    const PRE_SHIFT: u64 = 32;
    const TIME_BITS: u64 = Self::PRE_SHIFT + 20;
    const TIME_UNIT: u64 = 1 << Self::TIME_BITS;
    const FRAC_BITS: u64 = Self::TIME_BITS - Self::PRE_SHIFT;
    const PHASE_BITS: u64 = 5;
    const PHASE_SHIFT: u64 = Self::FRAC_BITS - Self::PHASE_BITS;
    const PHASE_COUNT: u32 = 1 << Self::PHASE_BITS;
    const DELTA_BITS: u64 = 15;
    const DELTA_UNIT: u32 = 1 << Self::DELTA_BITS;
    const BASS_SHIFT: u64 = 9;
    const MAX_SAMPLE: i32 = 32767;
    const BLIP_MAX_RATIO: u64 = 1 << 20;
    const HALF_WIDTH: u64 = 8;
    const END_FRAME_EXTRA: u64 = 2;
    const BUF_EXTRA: u64 = Self::HALF_WIDTH * 2 + Self::END_FRAME_EXTRA;

    pub fn new(clock_rate: f64, sample_rate: f64) -> Self {
        // TODO: check that S is higher than 0
        let factor = Self::TIME_UNIT as f64 * sample_rate / clock_rate;

        Self {
            factor: f64::ceil(factor) as u64,
            offset: (Self::TIME_UNIT / Self::BLIP_MAX_RATIO) / 2,
            available: 0,
            integrator: 0,
            buf: [0; S],
            last_sample: 0,
            time: 0,
        }
    }

    pub fn set_rates(&mut self, clock_rate: f64, sample_rate: f64) {
        let factor = Self::TIME_UNIT as f64 * sample_rate / clock_rate;
        self.factor = f64::ceil(factor) as u64;
    }

    const BL_STEP: [[i16; 8]; 33] = [
        [43, -115, 350, -488, 1136, -914, 5861, 21022],
        [44, -118, 348, -473, 1076, -799, 5274, 21001],
        [45, -121, 344, -454, 1011, -677, 4706, 20936],
        [46, -122, 336, -431, 942, -549, 4156, 20829],
        [47, -123, 327, -404, 868, -418, 3629, 20679],
        [47, -122, 316, -375, 792, -285, 3124, 20488],
        [47, -120, 303, -344, 714, -151, 2644, 20256],
        [46, -117, 289, -310, 634, -17, 2188, 19985],
        [46, -114, 273, -275, 553, 117, 1758, 19675],
        [44, -108, 255, -237, 471, 247, 1356, 19327],
        [43, -103, 237, -199, 390, 373, 981, 18944],
        [42, -98, 218, -160, 310, 495, 633, 18527],
        [40, -91, 198, -121, 231, 611, 314, 18078],
        [38, -84, 178, -81, 153, 722, 22, 17599],
        [36, -76, 157, -43, 80, 824, -241, 17092],
        [34, -68, 135, -3, 8, 919, -476, 16558],
        [32, -61, 115, 34, -60, 1006, -683, 16001],
        [29, -52, 94, 70, -123, 1083, -862, 15422],
        [27, -44, 73, 106, -184, 1152, -1015, 14824],
        [25, -36, 53, 139, -239, 1211, -1142, 14210],
        [22, -27, 34, 170, -290, 1261, -1244, 13582],
        [20, -20, 16, 199, -335, 1301, -1322, 12942],
        [18, -12, -3, 226, -375, 1331, -1376, 12293],
        [15, -4, -19, 250, -410, 1351, -1408, 11638],
        [13, 3, -35, 272, -439, 1361, -1419, 10979],
        [11, 9, -49, 292, -464, 1362, -1410, 10319],
        [9, 16, -63, 309, -483, 1354, -1383, 9660],
        [7, 22, -75, 322, -496, 1337, -1339, 9005],
        [6, 26, -85, 333, -504, 1312, -1280, 8355],
        [4, 31, -94, 341, -507, 1278, -1205, 7713],
        [3, 35, -102, 347, -506, 1238, -1119, 7082],
        [1, 40, -110, 350, -499, 1190, -1021, 6464],
        [0, 43, -115, 350, -488, 1136, -914, 5861],
    ];

    pub fn add_sample(&mut self, s: i32) {
        let delta = s - self.last_sample;
        self.last_sample = s;

        if delta != 0 {
            self.add_delta(delta);
        }

        self.time += 1;
    }

    fn add_delta(&mut self, mut delta: i32) {
        let fixed: u32 = ((self.time as u64 * self.factor + self.offset) >> Self::PRE_SHIFT) as u32;
        let phase = fixed >> Self::PHASE_SHIFT & (Self::PHASE_COUNT - 1);

        let interp: i32 =
            (fixed >> (Self::PHASE_SHIFT - Self::DELTA_BITS) & (Self::DELTA_UNIT - 1)) as i32;
        // TODO: correct shift ?
        let delta2 = (delta * interp) >> Self::DELTA_BITS;

        delta -= delta2;

        let s_in = Self::BL_STEP[phase as usize];
        let in_half_width = Self::BL_STEP[phase as usize + 1];
        let rev = Self::BL_STEP[(Self::PHASE_COUNT - phase) as usize];
        let rev_half_width = Self::BL_STEP[(Self::PHASE_COUNT - phase - 1) as usize];

        let buf_index = self.available as usize + (fixed as usize >> Self::FRAC_BITS as usize);

        for i in 0..8 {
            self.buf[(buf_index + i) as usize] +=
                s_in[i as usize] as i32 * delta + in_half_width[i as usize] as i32 * delta2;
        }

        for (i, j) in (8..16).zip((0..8).rev()) {
            self.buf[(buf_index + i) as usize] +=
                rev[j as usize] as i32 * delta + rev_half_width[j as usize] as i32 * delta2;
        }
    }

    /* fn add_delta_fast(&mut self, delta: i32) {
        let fixed: u32 = ((self.time as u64 * self.factor + self.offset) >> Self::PRE_SHIFT) as u32;

        let interp: i32 =
            (fixed >> (Self::FRAC_BITS - Self::DELTA_BITS) & (Self::DELTA_UNIT - 1)) as i32;
        let delta2 = delta * interp;

        let buf_index = self.available as usize + (fixed as usize >> Self::FRAC_BITS as usize);

        self.buf[buf_index + 7] += delta * Self::DELTA_UNIT as i32 - delta2;
        self.buf[buf_index + 8] += delta2;
    } */

    pub fn end_frame(&mut self, out: &mut Vec<i16>) {
        let off = self.time * self.factor + self.offset;
        self.available += off >> Self::TIME_BITS;
        self.offset = off & (Self::TIME_UNIT - 1);

        self.time = 0;

        self.read_samples(out)
    }

    fn read_samples(&mut self, out: &mut Vec<i16>) {
        let count = self.available as usize;

        if count > 0 {
            let mut sum = self.integrator;

            for i in 0..count {
                let s = sum >> Self::DELTA_BITS;
                sum += self.buf[i];

                let s_clamped = if (s as i16 as i32) != s {
                    (s >> 16) ^ Self::MAX_SAMPLE
                } else {
                    s
                };

                out.push(s_clamped as i16);

                sum -= s << (Self::DELTA_BITS - Self::BASS_SHIFT);
            }

            self.integrator = sum;

            let remain = (self.available + Self::BUF_EXTRA - count as u64) as usize;
            self.buf.copy_within(count..(count + remain), 0);
            self.buf[remain..(remain + count)].fill(0);

            self.available = 0;
        }
    }
}
