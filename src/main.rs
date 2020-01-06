use flate2::read::GzDecoder;
use std::fs::File;
use std::io::prelude::*;
use vgm::parser;

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

    let header = parser::header(&mut buffer).unwrap().1;

    dbg!(header);
}
