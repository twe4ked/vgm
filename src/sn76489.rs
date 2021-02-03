#![allow(
    dead_code,
    mutable_transmutes,
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    unused_assignments,
    unused_mut
)]

pub type int16_t = libc::c_short;
pub type int32_t = libc::c_int;
pub type uint32_t = libc::c_uint;

#[derive(Copy, Clone, Debug)]
#[repr(C)]
pub struct SNG {
    pub out: int32_t,
    pub clk: uint32_t,
    pub rate: uint32_t,
    pub base_incr: uint32_t,
    pub quality: uint32_t,
    pub count: [uint32_t; 3],
    pub volume: [uint32_t; 3],
    pub freq: [uint32_t; 3],
    pub edge: [uint32_t; 3],
    pub mute: [uint32_t; 3],
    pub noise_seed: uint32_t,
    pub noise_count: uint32_t,
    pub noise_freq: uint32_t,
    pub noise_volume: uint32_t,
    pub noise_mode: uint32_t,
    pub noise_fref: uint32_t,
    pub base_count: uint32_t,
    pub realstep: uint32_t,
    pub sngtime: uint32_t,
    pub sngstep: uint32_t,
    pub adr: uint32_t,
    pub stereo: uint32_t,
    pub ch_out: [int16_t; 4],
}

impl SNG {
    pub unsafe fn new(clock: uint32_t, rate: uint32_t) -> Self {
        let sng = transpiled::SNG_new(clock, rate);
        transpiled::SNG_reset(sng);
        *sng
    }

    pub unsafe fn set_rate(&mut self, rate: uint32_t) {
        transpiled::SNG_set_rate(self, rate);
    }

    pub unsafe fn write(&mut self, value: uint32_t) {
        transpiled::SNG_writeIO(self, value)
    }

    pub unsafe fn update(&mut self) {
        transpiled::SNG_calc(self);
    }
}

mod transpiled {
    // SN76489 emulator by Mitsutaka Okazaki 2001-2016
    //
    // 2001 08-13 : Version 1.00
    // 2001 10-03 : Version 1.01 -- Added SNG_set_quality().
    // 2004 05-23 : Version 1.10 -- Implemented GG stereo mode by RuRuRu
    // 2004 06-07 : Version 1.20 -- Improved the noise emulation.
    // 2015 12-13 : Version 1.21 -- Changed own integer types to C99 stdint.h types.
    // 2016 09-06 : Version 1.22 -- Support per-channel output.
    //
    // References:
    // SN76489 data sheet
    // sn76489.c   -- from MAME
    // sn76489.txt -- from http://www.smspower.org/
    #![allow(
        dead_code,
        mutable_transmutes,
        non_camel_case_types,
        non_snake_case,
        non_upper_case_globals,
        unused_assignments,
        unused_mut
    )]
    extern "C" {
        #[no_mangle]
        fn malloc(_: libc::c_ulong) -> *mut libc::c_void;
        #[no_mangle]
        fn free(_: *mut libc::c_void);
    }
    use super::*;

    pub type int16_t = libc::c_short;
    pub type int32_t = libc::c_int;
    pub type uint32_t = libc::c_uint;

    static mut voltbl: [uint32_t; 16] = [
        0xff as libc::c_int as uint32_t,
        0xcb as libc::c_int as uint32_t,
        0xa1 as libc::c_int as uint32_t,
        0x80 as libc::c_int as uint32_t,
        0x65 as libc::c_int as uint32_t,
        0x50 as libc::c_int as uint32_t,
        0x40 as libc::c_int as uint32_t,
        0x33 as libc::c_int as uint32_t,
        0x28 as libc::c_int as uint32_t,
        0x20 as libc::c_int as uint32_t,
        0x19 as libc::c_int as uint32_t,
        0x14 as libc::c_int as uint32_t,
        0x10 as libc::c_int as uint32_t,
        0xc as libc::c_int as uint32_t,
        0xa as libc::c_int as uint32_t,
        0 as libc::c_int as uint32_t,
    ];

    unsafe extern "C" fn internal_refresh(mut sng: *mut SNG) {
        if (*sng).quality != 0 {
            (*sng).base_incr = ((1 as libc::c_int) << 24 as libc::c_int) as uint32_t;
            (*sng).realstep = (((1 as libc::c_int) << 31 as libc::c_int) as libc::c_uint)
                .wrapping_div((*sng).rate);
            (*sng).sngstep = (((1 as libc::c_int) << 31 as libc::c_int) as libc::c_uint)
                .wrapping_div((*sng).clk.wrapping_div(16 as libc::c_int as libc::c_uint));
            (*sng).sngtime = 0 as libc::c_int as uint32_t
        } else {
            (*sng).base_incr = ((*sng).clk as libc::c_double
                * ((1 as libc::c_int) << 24 as libc::c_int) as libc::c_double
                / (16 as libc::c_int as libc::c_uint).wrapping_mul((*sng).rate) as libc::c_double)
                as uint32_t
        };
    }

    #[no_mangle]
    pub unsafe extern "C" fn SNG_set_rate(mut sng: *mut SNG, mut r: uint32_t) {
        (*sng).rate = if r != 0 {
            r
        } else {
            44100 as libc::c_int as libc::c_uint
        };
        internal_refresh(sng);
    }

    #[no_mangle]
    pub unsafe extern "C" fn SNG_set_quality(mut sng: *mut SNG, mut q: uint32_t) {
        (*sng).quality = q;
        internal_refresh(sng);
    }

    #[no_mangle]
    pub unsafe extern "C" fn SNG_new(mut c: uint32_t, mut r: uint32_t) -> *mut SNG {
        let mut sng: *mut SNG = 0 as *mut SNG;
        sng = malloc(::std::mem::size_of::<SNG>() as libc::c_ulong) as *mut SNG;
        if sng.is_null() {
            return 0 as *mut SNG;
        }
        (*sng).clk = c;
        (*sng).rate = if r != 0 {
            r
        } else {
            44100 as libc::c_int as libc::c_uint
        };
        SNG_set_quality(sng, 0 as libc::c_int as uint32_t);
        return sng;
    }

    #[no_mangle]
    pub unsafe extern "C" fn SNG_reset(mut sng: *mut SNG) {
        let mut i: libc::c_int = 0;
        (*sng).base_count = 0 as libc::c_int as uint32_t;
        i = 0 as libc::c_int;
        while i < 3 as libc::c_int {
            (*sng).count[i as usize] = 0 as libc::c_int as uint32_t;
            (*sng).freq[i as usize] = 0 as libc::c_int as uint32_t;
            (*sng).edge[i as usize] = 0 as libc::c_int as uint32_t;
            (*sng).volume[i as usize] = 0xf as libc::c_int as uint32_t;
            (*sng).mute[i as usize] = 0 as libc::c_int as uint32_t;
            i += 1
        }
        (*sng).adr = 0 as libc::c_int as uint32_t;
        (*sng).noise_seed = 0x8000 as libc::c_int as uint32_t;
        (*sng).noise_count = 0 as libc::c_int as uint32_t;
        (*sng).noise_freq = 0 as libc::c_int as uint32_t;
        (*sng).noise_volume = 0xf as libc::c_int as uint32_t;
        (*sng).noise_mode = 0 as libc::c_int as uint32_t;
        (*sng).noise_fref = 0 as libc::c_int as uint32_t;
        (*sng).out = 0 as libc::c_int;
        (*sng).stereo = 0xff as libc::c_int as uint32_t;
        (*sng).ch_out[3 as libc::c_int as usize] = 0 as libc::c_int as int16_t;
        (*sng).ch_out[2 as libc::c_int as usize] = (*sng).ch_out[3 as libc::c_int as usize];
        (*sng).ch_out[1 as libc::c_int as usize] = (*sng).ch_out[2 as libc::c_int as usize];
        (*sng).ch_out[0 as libc::c_int as usize] = (*sng).ch_out[1 as libc::c_int as usize];
    }

    #[no_mangle]
    pub unsafe extern "C" fn SNG_delete(mut sng: *mut SNG) {
        free(sng as *mut libc::c_void);
    }

    #[no_mangle]
    pub unsafe extern "C" fn SNG_writeIO(mut sng: *mut SNG, mut val: uint32_t) {
        if val & 0x80 as libc::c_int as libc::c_uint != 0 {
            (*sng).adr = (val & 0x70 as libc::c_int as libc::c_uint) >> 4 as libc::c_int;
            match (*sng).adr {
                0 | 2 | 4 => {
                    (*sng).freq[((*sng).adr >> 1 as libc::c_int) as usize] = (*sng).freq
                        [((*sng).adr >> 1 as libc::c_int) as usize]
                        & 0x3f0 as libc::c_int as libc::c_uint
                        | val & 0xf as libc::c_int as libc::c_uint
                }
                1 | 3 | 5 => {
                    (*sng).volume[((*sng).adr.wrapping_sub(1 as libc::c_int as libc::c_uint)
                        >> 1 as libc::c_int) as usize] = val & 0xf as libc::c_int as libc::c_uint
                }
                6 => {
                    (*sng).noise_mode =
                        (val & 4 as libc::c_int as libc::c_uint) >> 2 as libc::c_int;
                    if val & 0x3 as libc::c_int as libc::c_uint
                        == 0x3 as libc::c_int as libc::c_uint
                    {
                        (*sng).noise_freq = (*sng).freq[2 as libc::c_int as usize];
                        (*sng).noise_fref = 1 as libc::c_int as uint32_t
                    } else {
                        (*sng).noise_freq = ((32 as libc::c_int)
                            << (val & 0x3 as libc::c_int as libc::c_uint))
                            as uint32_t;
                        (*sng).noise_fref = 0 as libc::c_int as uint32_t
                    }
                    if (*sng).noise_freq == 0 as libc::c_int as libc::c_uint {
                        (*sng).noise_freq = 1 as libc::c_int as uint32_t
                    }
                    (*sng).noise_seed = 0x8000 as libc::c_int as uint32_t
                }
                7 => (*sng).noise_volume = val & 0xf as libc::c_int as libc::c_uint,
                _ => {}
            }
        } else {
            (*sng).freq[((*sng).adr >> 1 as libc::c_int) as usize] =
                (val & 0x3f as libc::c_int as libc::c_uint) << 4 as libc::c_int
                    | (*sng).freq[((*sng).adr >> 1 as libc::c_int) as usize]
                        & 0xf as libc::c_int as libc::c_uint
        };
    }

    #[inline]
    unsafe extern "C" fn parity(mut val: libc::c_int) -> libc::c_int {
        val ^= val >> 8 as libc::c_int;
        val ^= val >> 4 as libc::c_int;
        val ^= val >> 2 as libc::c_int;
        val ^= val >> 1 as libc::c_int;
        return val & 1 as libc::c_int;
    }

    #[inline]
    unsafe extern "C" fn update_output(mut sng: *mut SNG) {
        let mut i: libc::c_int = 0;
        let mut incr: uint32_t = 0;
        (*sng).base_count = ((*sng).base_count as libc::c_uint).wrapping_add((*sng).base_incr)
            as uint32_t as uint32_t;
        incr = (*sng).base_count >> 24 as libc::c_int;
        (*sng).base_count &=
            (((1 as libc::c_int) << 24 as libc::c_int) - 1 as libc::c_int) as libc::c_uint;
        /* Noise */
        (*sng).noise_count =
            ((*sng).noise_count as libc::c_uint).wrapping_add(incr) as uint32_t as uint32_t;
        if (*sng).noise_count & 0x100 as libc::c_int as libc::c_uint != 0 {
            if (*sng).noise_mode != 0 {
                /* White */
                (*sng).noise_seed = (*sng).noise_seed >> 1 as libc::c_int
                    | (parity(
                        ((*sng).noise_seed & 0x9 as libc::c_int as libc::c_uint) as libc::c_int,
                    ) << 15 as libc::c_int) as libc::c_uint
            } else {
                /* Periodic */
                (*sng).noise_seed = (*sng).noise_seed >> 1 as libc::c_int
                    | ((*sng).noise_seed & 1 as libc::c_int as libc::c_uint) << 15 as libc::c_int
            }
            if (*sng).noise_fref != 0 {
                (*sng).noise_count = ((*sng).noise_count as libc::c_uint)
                    .wrapping_sub((*sng).freq[2 as libc::c_int as usize])
                    as uint32_t as uint32_t
            } else {
                (*sng).noise_count = ((*sng).noise_count as libc::c_uint)
                    .wrapping_sub((*sng).noise_freq)
                    as uint32_t as uint32_t
            }
        }
        if (*sng).noise_seed & 1 as libc::c_int as libc::c_uint != 0 {
            (*sng).ch_out[3 as libc::c_int as usize] = ((*sng).ch_out[3 as libc::c_int as usize]
                as libc::c_uint)
                .wrapping_add(voltbl[(*sng).noise_volume as usize] << 4 as libc::c_int)
                as int16_t as int16_t
        }
        (*sng).ch_out[3 as libc::c_int as usize] = ((*sng).ch_out[3 as libc::c_int as usize]
            as libc::c_int
            >> 1 as libc::c_int) as int16_t;
        /* Tone */
        i = 0 as libc::c_int;
        while i < 3 as libc::c_int {
            (*sng).count[i as usize] = ((*sng).count[i as usize] as libc::c_uint).wrapping_add(incr)
                as uint32_t as uint32_t;
            if (*sng).count[i as usize] & 0x400 as libc::c_int as libc::c_uint != 0 {
                if (*sng).freq[i as usize] > 1 as libc::c_int as libc::c_uint {
                    (*sng).edge[i as usize] =
                        ((*sng).edge[i as usize] == 0) as libc::c_int as uint32_t;
                    (*sng).count[i as usize] = ((*sng).count[i as usize] as libc::c_uint)
                        .wrapping_sub((*sng).freq[i as usize])
                        as uint32_t as uint32_t
                } else {
                    (*sng).edge[i as usize] = 1 as libc::c_int as uint32_t
                }
            }
            if (*sng).edge[i as usize] != 0 && (*sng).mute[i as usize] == 0 {
                (*sng).ch_out[i as usize] = ((*sng).ch_out[i as usize] as libc::c_uint)
                    .wrapping_add(voltbl[(*sng).volume[i as usize] as usize] << 4 as libc::c_int)
                    as int16_t as int16_t
            }
            (*sng).ch_out[i as usize] =
                ((*sng).ch_out[i as usize] as libc::c_int >> 1 as libc::c_int) as int16_t;
            i += 1
        }
    }

    #[inline]
    unsafe extern "C" fn mix_output(mut sng: *mut SNG) -> int16_t {
        (*sng).out = (*sng).ch_out[0 as libc::c_int as usize] as libc::c_int
            + (*sng).ch_out[1 as libc::c_int as usize] as libc::c_int
            + (*sng).ch_out[2 as libc::c_int as usize] as libc::c_int
            + (*sng).ch_out[3 as libc::c_int as usize] as libc::c_int;
        return (*sng).out as int16_t;
    }

    #[no_mangle]
    pub unsafe extern "C" fn SNG_calc(mut sng: *mut SNG) -> int16_t {
        if (*sng).quality == 0 {
            update_output(sng);
            return mix_output(sng);
        }
        /* Simple rate converter */
        while (*sng).realstep > (*sng).sngtime {
            (*sng).sngtime = ((*sng).sngtime as libc::c_uint).wrapping_add((*sng).sngstep)
                as uint32_t as uint32_t;
            update_output(sng);
        }
        (*sng).sngtime = (*sng).sngtime.wrapping_sub((*sng).realstep);
        return mix_output(sng);
    }

    #[inline]
    unsafe extern "C" fn mix_output_stereo(mut sng: *mut SNG, mut out: *mut int32_t) {
        let mut i: libc::c_int = 0;
        let ref mut fresh0 = *out.offset(1 as libc::c_int as isize);
        *fresh0 = 0 as libc::c_int;
        *out.offset(0 as libc::c_int as isize) = *fresh0;
        if (*sng).stereo >> 4 as libc::c_int & 0x8 as libc::c_int as libc::c_uint != 0 {
            let ref mut fresh1 = *out.offset(0 as libc::c_int as isize);
            *fresh1 += (*sng).ch_out[3 as libc::c_int as usize] as libc::c_int
        }
        if (*sng).stereo & 0x8 as libc::c_int as libc::c_uint != 0 {
            let ref mut fresh2 = *out.offset(1 as libc::c_int as isize);
            *fresh2 += (*sng).ch_out[3 as libc::c_int as usize] as libc::c_int
        }
        i = 0 as libc::c_int;
        while i < 3 as libc::c_int {
            if (*sng).stereo >> i + 4 as libc::c_int & 0x1 as libc::c_int as libc::c_uint != 0 {
                let ref mut fresh3 = *out.offset(0 as libc::c_int as isize);
                *fresh3 += (*sng).ch_out[i as usize] as libc::c_int
            }
            if (*sng).stereo >> i & 0x1 as libc::c_int as libc::c_uint != 0 {
                let ref mut fresh4 = *out.offset(1 as libc::c_int as isize);
                *fresh4 += (*sng).ch_out[i as usize] as libc::c_int
            }
            i += 1
        }
    }

    #[no_mangle]
    pub unsafe extern "C" fn SNG_calc_stereo(mut sng: *mut SNG, mut out: *mut int32_t) {
        if (*sng).quality == 0 {
            update_output(sng);
            mix_output_stereo(sng, out);
            return;
        }
        while (*sng).realstep > (*sng).sngtime {
            (*sng).sngtime = ((*sng).sngtime as libc::c_uint).wrapping_add((*sng).sngstep)
                as uint32_t as uint32_t;
            update_output(sng);
        }
        (*sng).sngtime = (*sng).sngtime.wrapping_sub((*sng).realstep);
        mix_output_stereo(sng, out);
    }

    #[no_mangle]
    pub unsafe extern "C" fn SNG_writeGGIO(mut sng: *mut SNG, mut val: uint32_t) {
        (*sng).stereo = val;
    }
}

pub fn play(buffer: &[u8]) {
    let mut i = 0;
    let mut chip = unsafe { SNG::new(3579545, 0) };
    unsafe { chip.set_rate(0) };
    dbg!(chip);

    loop {
        match buffer[i] {
            0x50 => {
                // Write value dd.
                unsafe {
                    chip.write(buffer[i + 1] as u32);
                    chip.update();
                }
                dbg!(chip.out);
                i += 1;
            }
            0x61 => {
                // Wait n samples, n can range from 0 to 65535 (approx 1.49 seconds). Longer
                // pauses than this are represented by multiple wait commands.
                // println!("0x61 {:#04x} {:#04x}", buffer[i + 1], buffer[i + 2]);
                i += 2;
            }
            0x62 => {
                // Wait 735 samples (60th of a second), a shortcut for 0x61 0xdf 0x02.
            }
            0x70..=0x7f => {
                // Wait n+1 samples, n can range from 0 to 15.
                // println!("0x7x {:#04x}", buffer[i] & 0x0f);
            }
            0x66 => {
                // End of sound data.
                println!("end");
                // break;
            }
            _ => unimplemented!("{:#04x}", buffer[i]),
        }
        i += 1;
    }

    dbg!(&buffer[i..].len());
}
