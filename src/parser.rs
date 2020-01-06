use crate::header::{Header, SN76489Feedback, SN76489Flags, SN76489ShiftRegisterWidth, SN76489};
use byteorder::{ByteOrder, LittleEndian};
use nom::bytes::complete::{tag, take};
use nom::IResult;
use nom_locate::LocatedSpan;

type Span<'a> = LocatedSpan<&'a [u8]>;

fn take_u8(input: Span) -> IResult<Span, u8> {
    let (input, output) = take(1u8)(input)?;
    Ok((input, output.fragment[0]))
}

fn take_u16(input: Span) -> IResult<Span, u16> {
    let (input, output) = take(2u8)(input)?;
    Ok((input, LittleEndian::read_u16(output.fragment)))
}

fn take_u32(input: Span) -> IResult<Span, u32> {
    let (input, output) = take(4u8)(input)?;
    Ok((input, LittleEndian::read_u32(output.fragment)))
}

fn take_option_u32(input: Span) -> IResult<Span, Option<u32>> {
    let (input, output) = take(4u8)(input)?;
    let output = LittleEndian::read_u32(output.fragment);
    if output == 0 {
        Ok((input, None))
    } else {
        Ok((input, Some(output)))
    }
}

// https://vgmrips.net/wiki/VGM_Specification
pub fn header(input: &[u8]) -> IResult<Span, Header> {
    let input = Span::new(input);

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
        Some(SN76489::new(
            sn76489_clock,
            sn76489_feedback,
            sn76489_shift_register_width,
            sn76489_flags,
        ))
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

    let current_position = input.offset;

    // VGM 1.50 additions:
    let (input, data_offset) = take_u32(input)?;
    // If the VGM data starts at absolute offset 0x40, this will contain value 0x0000000C. For
    // versions prior to 1.50, it should be 0 and the VGM data must start at offset 0x40.
    let mut data_offset = if version < 0x00000150 {
        0x0000000c
    } else {
        data_offset
    };
    // Add our current position in the header. If we have 0x0000000c + 0x00000034 we'll get 0x40.
    data_offset += current_position as u32; // _ + 0x00000034

    // From here on we might be reading non-header data.
    //
    // All header sizes are valid for all versions from 1.50 on, as long as header has at least 64
    // bytes. If the VGM data starts at an offset that is lower than 0x100, all overlapping header
    // bytes have to be handled as they were zero.
    //
    // TODO: Implement overlap check using input.offset.

    // VGM 1.51 additions:
    let (input, sega_pcm_clock) = take_u32(input)?;
    let sega_pcm_clock = if version < 0x00000151 {
        None
    } else {
        Some(sega_pcm_clock)
    };
    let (input, spcm_interface) = take_u32(input)?;
    let spcm_interface = if version < 0x00000151 {
        None
    } else {
        Some(spcm_interface)
    };

    Ok((
        input,
        Header {
            eof_offset,
            version,
            sn76489,
            ym2413_clock,
            gd3_offset,
            total_samples,
            loop_offset,
            loop_samples,
            rate,
            ym2612_clock,
            ym2151_clock,
            data_offset,
            sega_pcm_clock,
            spcm_interface,
        },
    ))
}
