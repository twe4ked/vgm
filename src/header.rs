use std::fmt;

#[derive(CustomDebug)]
pub struct SN76489 {
    /// Input clock rate in Hz for the SN76489 PSG chip. A typical value is 3579545.
    /// It should be None if there is no PSG chip used.
    ///
    /// TODO: Note: Bit 31 (0x80000000) is used on combination with the dual-chip-bit to indicate
    /// that this is a T6W28. (PSG variant used in Neo Geo Pocket)
    #[debug(with = "u32_hex_fmt")]
    pub clock: u32,
    pub feedback: Option<SN76489Feedback>,
    pub shift_register_width: Option<SN76489ShiftRegisterWidth>,
    pub flags: Option<SN76489Flags>,
    pub t6w28: bool,
    pub dual_chip_bit: bool,
}

impl SN76489 {
    pub fn new(
        clock: u32,
        feedback: Option<SN76489Feedback>,
        shift_register_width: Option<SN76489ShiftRegisterWidth>,
        flags: Option<SN76489Flags>,
    ) -> Self {
        Self {
            clock,
            feedback,
            shift_register_width,
            flags,
            t6w28: clock & 0x80000000 != 0,
            dual_chip_bit: clock & 0x40000000 != 0,
        }
    }
}

bitflags! {
    /// The white noise feedback pattern for the SN76489 PSG.
    ///
    /// For version 1.01 and earlier files, the feedback pattern should be assumed to be 0x0009. If
    /// the PSG is not used then this may be omitted (left at zero).
    pub struct SN76489Feedback: u16 {
        /// 0x0009: Sega Master System 2/Game Gear/Mega Drive (SN76489/SN76496 integrated into Sega VDP chip)
        const NINE = 0x0009;

        /// 0x0003: Sega Computer 3000H, BBC Micro (SN76489AN)
        const THREE = 0x0003;

        /// 0x0006: SN76494, SN76496
        const SIX = 0x0006;
    }
}

bitflags! {
    /// The noise feedback shift register width, in bits. Known values are:
    ///
    /// For version 1.01 and earlier files, the shift register width should be assumed to be 16. If
    /// the PSG is not used then this may be omitted (left at zero).
    pub struct SN76489ShiftRegisterWidth: u8 {
        /// 0x16:   Sega Master System 2/Game Gear/Mega Drive (SN76489/SN76496 integrated into Sega
        ///         VDP chip)
        const SIXTEEN = 0x16;

        /// 0x15:   Sega Computer 3000H, BBC Micro (SN76489AN)
        const FIFTEEN = 0x15;
    }
}

bitflags! {
    /// SN76489 Flags
    ///
    /// Misc flags for the SN76489. Most of them don't make audible changes and can be ignored, if the
    /// SN76489 emulator lacks the features.
    ///
    /// For version 1.51 and earlier files, all the flags should not be set. If the PSG is not used
    /// then this may be omitted (left at zero).
    pub struct SN76489Flags: u8 {
        /// bit 0: frequency 0 is 0x400
        const FREQUENCY_0_IS_0X400 = 0b00000001;

        /// bit 1: output negate flag
        const OUTPUT_NEGATE_FLAG = 0b00000010;

        /// bit 2: stereo on/off (on when bit clear)
        const STEREO_ON_OFF = 0b00000100;

        /// bit 3: /8 Clock Divider on/off (on when bit clear)
        const CLOCK_DIVIDER_ON_OFF = 0b00001000;
    }
}

#[derive(CustomDebug)]
pub struct Header {
    /// Relative offset to end of file (i.e. file length - 4). This is mainly used to find the next
    /// track when concatenating player stubs and multiple files.
    #[debug(with = "u32_hex_fmt")]
    pub eof_offset: u32,

    /// Version number in BCD-Code. e.g. Version 1.70 is stored as 0x00000171. This is used for
    /// backwards compatibility in players, and defines which header values are valid.
    #[debug(with = "u32_hex_fmt")]
    pub version: u32,

    /// SN76489
    pub sn76489: Option<SN76489>,

    /// Input clock rate in Hz for the YM2413 chip. A typical value is 3579545.
    /// It should be None if there is no YM2413 chip used.
    #[debug(with = "option_u32_hex_fmt")]
    pub ym2413_clock: Option<u32>,

    /// Relative offset to GD3 tag. 0 if no GD3 tag. GD3 tags are descriptive tags similar in use
    /// to ID3 tags in MP3 files. See the GD3 specification for more details. The GD3 tag is
    /// usually stored immediately after the VGM data.
    #[debug(with = "option_u32_hex_fmt")]
    pub gd3_offset: Option<u32>,

    /// Total # samples: Total of all wait values in the file.
    #[debug(with = "u32_hex_fmt")]
    pub total_samples: u32,

    /// Loop offset: Relative offset to loop point, or 0 if no loop. For example, if the data for
    /// the one-off intro to a song was in bytes 0x0040 - 0x3FFF of the file, but the main looping
    /// section started at 0x4000, this would contain the value 0x4000 - 0x1C = 0x00003FE4.
    #[debug(with = "u32_hex_fmt")]
    pub loop_offset: u32,

    /// Number of samples in one loop, or 0 if there is no loop. Total of all wait values between
    /// the loop point and the end of the file.
    #[debug(with = "u32_hex_fmt")]
    pub loop_samples: u32,

    /// "Rate" of recording in Hz, used for rate scaling on playback. It is typically 50 for PAL
    /// systems and 60 for NTSC systems. It should be set to zero if rate scaling is not
    /// appropriate - for example, if the game adjusts its music engine for the system's speed.
    ///
    /// VGM 1.00 files will have a value of 0.
    #[debug(with = "option_u32_hex_fmt")]
    pub rate: Option<u32>,

    /// Input clock rate in Hz for the YM2612 chip. A typical value is 7670454.
    ///
    /// It should be 0 if there us no YM2612 chip used.
    ///
    /// For version 1.01 and earlier files, the YM2413 clock rate should be used for the clock rate
    /// of the YM2612.
    #[debug(with = "option_u32_hex_fmt")]
    pub ym2612_clock: Option<u32>,

    /// Input clock rate in Hz for the YM2151 chip. A typical value is 3579545.
    ///
    /// It should be 0 if there us no YM2151 chip used.
    ///
    /// For version 1.01 and earlier files, the YM2413 clock rate should be used for the clock rate
    /// of the YM2151.
    #[debug(with = "option_u32_hex_fmt")]
    pub ym2151_clock: Option<u32>,

    /// VGM data offset
    ///
    /// Relative offset to VGM data stream.
    ///
    /// If the VGM data starts at absolute offset 0x40, this will contain value 0x0000000C.
    #[debug(with = "u32_hex_fmt")]
    pub data_offset: u32,
}

fn u32_hex_fmt<T: fmt::Debug + fmt::LowerHex>(n: &T, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:#010x}", n)
}

fn option_u32_hex_fmt<T: fmt::Debug + fmt::LowerHex>(
    n: &Option<T>,
    f: &mut fmt::Formatter,
) -> fmt::Result {
    if let Some(n) = n {
        u32_hex_fmt(n, f)
    } else {
        write!(f, "None")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sn76489_new() {
        fn new(clock: u32) -> SN76489 {
            let feedback = Some(SN76489Feedback::from_bits_truncate(0));
            let shift_register_width = Some(SN76489ShiftRegisterWidth::from_bits_truncate(0));
            let flags = Some(SN76489Flags::from_bits_truncate(0));

            SN76489::new(clock, feedback, shift_register_width, flags)
        }

        let clock = 0x80000000 | 42;
        assert!(new(clock).t6w28);
        assert!(!new(clock).dual_chip_bit);

        let clock = 0x40000000 | 42;
        assert!(!new(clock).t6w28);
        assert!(new(clock).dual_chip_bit);

        let clock = 0x40000000 | 0x80000000 | 42;
        assert!(new(clock).t6w28);
        assert!(new(clock).dual_chip_bit);
    }
}
