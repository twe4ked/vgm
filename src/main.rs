use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use vgm::parser;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let filename = args.get(1).expect("no file provided");
    let f = File::open(&filename).unwrap();
    let mut reader = BufReader::new(f);
    let mut header = [0; 256];
    reader.read_exact(&mut header).unwrap();

    let header = parser::header(&header).unwrap().1;

    dbg!(header);
}
