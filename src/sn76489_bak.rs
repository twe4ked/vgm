// #[derive(PartialEq, Debug)]
// enum Register {
//     Noise,
//     Tone(u8),
//     Volume(u8),
// }
//
// impl Register {
//     fn decode(byte: u8) -> Self {
//         let c = (byte >> 5) & 0b0000_0011; // 0cc0_0000
//         let t = (byte >> 4) & 0b0000_0001; // 000t_0000
//
//         if t == 0 {
//             if c == 3 {
//                 Register::Noise
//             } else {
//                 Register::Tone(c)
//             }
//         } else {
//             Register::Volume(c)
//         }
//     }
// }
//
// // https://www.smspower.org/Development/SN76489
// // https://github.com/vgmrips/vgmplay/blob/master/VGMPlay/chips/sn76489.c
// //
// // The SN76489 has 8 "registers" - 4 x 4 bit volume registers, 3 x 10 bit tone registers and 1 x 3
// // bit noise register. Of course, for hardware reasons these may internally be wider.
// #[derive(Debug)]
// struct SN76489 {
//     // Volume registers: The value represents the attenuation of the output. Hence, %0000 is full
//     // volume and %1111 is silence.
//     //
//     // Tone registers: These give a counter reset value for the tone generators. Hence, low values
//     // give high frequencies and vice versa.
//     //
//     // Noise register: One bit selects the mode ("periodic" or "white") and the other two select a
//     // shift rate.
//     channels: [[u8; 2]; 4],
//     // ...
//     // channel_1_counter: u16, // 10 bit
//     // channel_1_output: bool, // 1 bit
// }
//
// impl SN76489 {
//     // It appears the initial state of these registers depends on the hardware:
//     //
//     // -    Discrete chips seem to start with random values (an SC-3000 is reported to start with a
//     //      tone before the chip is written to by the software).
//     // -    The Sega integrated versions seem to start initialised with zeroes in the tone/noise
//     //      registers and ones in the volume registers (silence).
//     //
//     //      Channel     Tone & noise registers  Volume
//     //      0           Tone0                   Vol0
//     //      1           Tone1                   Vol1
//     //      2           Tone2                   Vol2
//     //      3           Noise                   Vol3
//     fn new() -> Self {
//         Self {
//             channels: [[0, 0b1111], [0, 0b1111], [0, 0b1111], [0, 0b1111]],
//         }
//     }
//
//     fn write(&mut self, byte: u8) {
//         if byte & 0b1000_0000 != 0 {
//             // If bit 7 is 1 then the byte is a LATCH/DATA byte:
//             //
//             //      %1cctdddd
//             //        |||````-- Data
//             //        ||`------ Type
//             //        ``------- Channel
//             //
//             //      Bits 6 and 5 (cc) give the channel to be latched, ALWAYS. This selects the row
//             //      in the above table - %00 is channel 0, %01 is channel 1, %10 is channel 2, %11
//             //      is channel 3 as you might expect.
//             //
//             //      Bit 4 (t) determines whether to latch volume (1) or tone/noise (0) data - this
//             //      gives the column.
//             //
//             //      The remaining 4 bits (dddd) are placed into the low 4 bits of the relevant
//             //      register. For the three-bit noise register, the highest bit is discarded.
//             //
//             //      The latched register is NEVER cleared by a data byte.
//             let register = Register::decode(byte);
//             let data = byte & 0b0000_1111; // 0000_dddd
//
//             match register {
//                 Register::Noise => self.channels[3][0] = data,
//                 Register::Tone(channel) => self.channels[channel as usize][0] = data,
//                 Register::Volume(channel) => self.channels[channel as usize][1] = data,
//             }
//         } else {
//             // If bit 7 is 0 then the byte is a DATA byte:
//             //
//             //        %0-DDDDDD
//             //          |``````-- Data
//             //          `-------- Unused
//             //
//             //      If the currently latched register is a tone register then the low 6 bits of the
//             //      byte (DDDDDD) are placed into the high 6 bits of the latched register. If the
//             //      latched register is less than 6 bits wide (ie. not one of the tone registers),
//             //      instead the low bits are placed into the corresponding bits of the register,
//             //      and any extra high bits are discarded.
//             //
//             //      The data have the following meanings (described more fully later):
//             //
//             //      Tone registers
//             //          DDDDDDdddd = cccccccccc
//             //
//             //          DDDDDDdddd gives the 10-bit half-wave counter reset value.
//             //
//             //      Volume registers
//             //          (DDDDDD)dddd = (--vvvv)vvvv
//             //
//             //          dddd gives the 4-bit volume value.
//             //          If a data byte is written, the low 4 bits of DDDDDD update the 4-bit volume value. However, this is unnecessary.
//             //
//             //      Noise register
//             //          (DDDDDD)dddd = (---trr)-trr
//             //
//             //          The low 2 bits of dddd select the shift rate and the next highest bit (bit 2) selects the mode (white (1) or "periodic" (0)).
//             //          If a data byte is written, its low 3 bits update the shift rate and mode in the same way.
//             //
//             //      This means that the following data will have the following effect (spacing added for clarity, hopefully):
//             //
//             //       %1 00 0 1110      Latch, channel 0, tone, data %1110
//             //       %0 0  001111      Data %001111
//             //
//             //      Set channel 0 tone to %0011111110 = 0xfe (440Hz @ 3579545Hz clock)
//             //
//             //       %1 01 1 1111      Latch, channel 1, volume, data %1111
//             //
//             //      Set channel 1 volume to %1111 = 0xf (silent)
//             //
//             //       %1 10 1 1111      Latch, channel 2, volume, data %1111
//             //       %0 0  000000      Data %000000
//             //
//             //      Set channel 2 volume to %1111 = 0xf (silent) THEN update it to %0000 = 0x0 (full) The data byte is NOT ignored. If it is, you will hear a sustained tone while reading a message box in Alex Kidd in Miracle World.
//             //
//             //       %1 11 0 0101      Latch, channel 3, noise, data %0101
//             //
//             //      Set noise register to %101 (white noise, medium shift rate)
//             //
//             //       %1 11 0 0101      Latch, channel 3, noise, data %0101
//             //       %0 0  000100      Data %000100
//             //
//             //      Set noise register to %101 (white noise, medium shift rate) THEN update it to %100 (white noise, high shift rate) The data byte is NOT ignored. If it is, some games (e.g. Micro Machines) produce the wrong sound on their noise channel.
//             //
//             //      Also of note is that the tone registers update immediately when a byte is written; they do not wait until all 10 bits are written.
//             //      Data written	Tone0 contents
//             //      1 00 0 0000	------0000
//             //      0 0 000000	0000000000
//             //      1 00 0 1111	0000001111
//             //      0 0 111111	1111111111
//             //
//             //      - signifies an unknown bit (whatever was previously in the register)
//             //
//             //      There were a couple of ways to handle SN76489 writes in older, inaccurate emulators:
//             //
//             //          Latch only the tone registers, as above, and leave them latched when other types of data (volume, noise) are written. This gives a "squawk" effect on SMS Micro Machines' title screen, which drowns out the "eek".
//             //          Latch tone registers as above, and "unlatch" when other types of data are written. When a data byte is written with it unlatched, the data is discarded. This fixes the "squawk" but leaves the "eek".
//             let data = byte & 0b0011_1111; // 00dd_dddd
//         }
//     }
//
//     fn volume(&self, channel: usize) -> u8 {
//         assert!(channel <= 3, "channel must be between 0 and 3");
//         self.channels[channel][1]
//     }
//
//     fn tone(&self, channel: usize) -> u8 {
//         assert!(channel < 3, "channel must be between 0 and 2");
//         self.channels[channel][0]
//     }
//
//     fn noise(&self) -> u8 {
//         self.channels[3][0]
//     }
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test_register_decode() {
//         assert_eq!(Register::decode(0b0_11_1_0000), Register::Volume(3));
//         //                              ^^ ^
//         //                               | `Volume
//         //                               `Channel 3
//
//         assert_eq!(Register::decode(0b0_10_0_0000), Register::Tone(2));
//         //                              ^^ ^
//         //                               | `Tone
//         //                               `Channel 2
//         //
//
//         assert_eq!(Register::decode(0b0_11_0_0000), Register::Noise);
//         //                              ^^ ^
//         //                               | `Noise
//         //                               `Channel 3
//     }
//
//     #[test]
//     fn test_latch_data() {
//         let mut chip = SN76489::new();
//
//         chip.write(0b1_01_1_0010);
//         //           ^ ^^ ^ ^^^^
//         //           |  | |    `2
//         //           |  |  `Volume
//         //           |   `Channel 1
//         //           `LATCH/DATA
//         assert_eq!(chip.volume(1), 2);
//
//         chip.write(0b1_10_0_1001);
//         //           ^ ^^ ^ ^^^^
//         //           |  | |    `9
//         //           |  |  `Tone
//         //           |   `Channel 1
//         //           `LATCH/DATA
//         assert_eq!(chip.tone(2), 9);
//
//         chip.write(0b1_11_0_1010);
//         //           ^ ^^ ^ ^^^^
//         //           |  | |    `10
//         //           |  |  `Tone
//         //           |   `Channel 3
//         //           `LATCH/DATA
//         assert_eq!(chip.noise(), 10);
//     }
// }
