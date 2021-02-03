use flate2::read::GzDecoder;
use std::fs::File;
use std::io::prelude::*;
use vgm::{parser, sn76489};

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let filename = args.get(1).expect("no file provided");
    let mut f = File::open(&filename).unwrap();
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).unwrap();

    let mut gz = GzDecoder::new(&buffer[..]);
    let mut buffer = if let Some(_) = gz.header() {
        let mut buffer = Vec::new();
        gz.read_to_end(&mut buffer).unwrap();
        buffer
    } else {
        buffer
    };

    let (span, header) = parser::header(&mut buffer).unwrap();

    dbg!(&header);

    let buffer = span.fragment;

    if let Some(_sn76489) = header.sn76489 {
        sn76489::play(&buffer);
    } else {
        unimplemented!();
    }
}

// fn audio() {
//     use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
//
//     let host = cpal::default_host();
//     let device = host
//         .default_output_device()
//         .expect("failed to find a default output device");
//     let format = device
//         .default_output_format()
//         .expect("failed to get default output format");
//     let event_loop = host.event_loop();
//     let stream_id = event_loop
//         .build_output_stream(&device, &format)
//         .expect("failed to build output stream");
//     event_loop
//         .play_stream(stream_id.clone())
//         .expect("failed to play stream");
//
//     let sample_rate = format.sample_rate.0 as f32;
//     let mut sample_clock = 0f32;
//
//     // Produce a sinusoid of maximum amplitude.
//     let mut next_value = || {
//         sample_clock = (sample_clock + 1.0) % sample_rate;
//         (sample_clock * 440.0 * 2.0 * 3.141592 / sample_rate).sin()
//     };
//
//     event_loop.run(move |id, result| {
//         let data = match result {
//             Ok(data) => data,
//             Err(err) => {
//                 eprintln!("an error occurred on stream {:?}: {}", id, err);
//                 return;
//             }
//         };
//
//         match data {
//             cpal::StreamData::Output {
//                 buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer),
//             } => {
//                 for sample in buffer.chunks_mut(format.channels as usize) {
//                     let value = ((next_value() * 0.5 + 0.5) * std::u16::MAX as f32) as u16;
//                     for out in sample.iter_mut() {
//                         *out = value;
//                     }
//                 }
//             }
//             cpal::StreamData::Output {
//                 buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer),
//             } => {
//                 for sample in buffer.chunks_mut(format.channels as usize) {
//                     let value = (next_value() * std::i16::MAX as f32) as i16;
//                     for out in sample.iter_mut() {
//                         *out = value;
//                     }
//                 }
//             }
//             cpal::StreamData::Output {
//                 buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
//             } => {
//                 for sample in buffer.chunks_mut(format.channels as usize) {
//                     let value = next_value();
//                     for out in sample.iter_mut() {
//                         *out = value;
//                     }
//                 }
//             }
//             _ => (),
//         }
//     });
// }
