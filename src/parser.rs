use crate::header::{Header, SN76489Feedback, SN76489Flags, SN76489ShiftRegisterWidth, SN76489};
use byteorder::{ByteOrder, LittleEndian};
use nom::bytes::complete::{tag, take};
use nom::IResult;

fn take_u8(input: &[u8]) -> IResult<&[u8], u8> {
    let (input, output) = take(1u8)(input)?;
    Ok((input, output[0]))
}

fn take_u16(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, output) = take(2u8)(input)?;
    Ok((input, LittleEndian::read_u16(output)))
}

fn take_u32(input: &[u8]) -> IResult<&[u8], u32> {
    let (input, output) = take(4u8)(input)?;
    Ok((input, LittleEndian::read_u32(output)))
}

fn take_option_u32(input: &[u8]) -> IResult<&[u8], Option<u32>> {
    let (input, output) = take(4u8)(input)?;
    let output = LittleEndian::read_u32(output);
    if output == 0 {
        Ok((input, None))
    } else {
        Ok((input, Some(output)))
    }
}

// https://vgmrips.net/wiki/VGM_Specification
pub fn header(input: &[u8]) -> IResult<&[u8], Header> {
    // File identification "Vgm " (0x56 0x67 0x6d 0x20)
    let (input, _ident) = tag("Vgm ")(input)?;
    let (input, eof_offset) = take_u32(input)?;
    let (input, version) = take_u32(input)?;
    let (input, sn76489_clock) = take_u32(input)?;
    let (input, ym2413_clock) = take_option_u32(input)?;
    let (input, gd3_offset) = take_option_u32(input)?;
    let (input, total_samples) = take_u32(input)?;
    let (input, loop_offset) = take_u32(input)?;
    let (input, loop_samples) = take_u32(input)?;

    // VGM 1.01 additions:
    let (input, rate) = take_u32(input)?;
    let rate = if version < 0x00000101 {
        None
    } else {
        Some(rate)
    };

    // VGM 1.10 additions:
    let (input, sn76489_feedback) = take_u16(input)?;
    let sn76489_feedback = if version < 0x00000110 {
        None
    } else {
        Some(SN76489Feedback::from_bits_truncate(sn76489_feedback))
    };
    let (input, sn76489_shift_register_width) = take_u8(input)?;
    let sn76489_shift_register_width = if version < 0x00000110 {
        None
    } else {
        Some(SN76489ShiftRegisterWidth::from_bits_truncate(
            sn76489_shift_register_width,
        ))
    };

    // VGM 1.51 additions:
    let (input, sn76489_flags) = take_u8(input)?;
    let sn76489_flags = if version < 0x00000151 {
        None
    } else {
        Some(SN76489Flags::from_bits_truncate(sn76489_flags))
    };

    let sn76489 = if sn76489_clock == 0 {
        None
    } else {
        Some(SN76489 {
            clock: sn76489_clock,
            feedback: sn76489_feedback,
            shift_register_width: sn76489_shift_register_width,
            flags: sn76489_flags,
            t6w28: sn76489_clock & 0x80000000 == 1,
            dual_chip_bit: sn76489_clock & 0x40000000 == 1,
        })
    };

    // VGM 1.10 additions:
    let (input, ym2612_clock) = take_u32(input)?;
    let ym2612_clock = if version < 0x00000110 {
        None
    } else {
        Some(ym2612_clock)
    };
    let (input, ym2151_clock) = take_u32(input)?;
    let ym2151_clock = if version < 0x00000110 {
        None
    } else {
        Some(ym2151_clock)
    };

    // VGM 1.50 additions:
    let (input, data_offset) = take_u32(input)?;
    // For versions prior to 1.50, it should be 0 and the VGM data must start at offset 0x40.
    let data_offset = if version < 0x00000150 {
        0x40
    } else {
        data_offset
    };

    Ok((
        input,
        Header {
            eof_offset: eof_offset,
            version: version,
            sn76489: sn76489,
            ym2413_clock: ym2413_clock,
            gd3_offset: gd3_offset,
            total_samples: total_samples,
            loop_offset: loop_offset,
            loop_samples: loop_samples,
            rate: rate,
            ym2612_clock: ym2612_clock,
            ym2151_clock: ym2151_clock,
            data_offset: data_offset,
        },
    ))
}
