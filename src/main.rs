use playfield::{Coordinate, Playfield};

fn main() {
    println!("Hello, world!");

    let bytes: Vec<Vec<u8>> = vec![vec![0x01, 0x02], vec![0x13, 0x14], vec![0x25, 0x26]];

    let _p = Playfield::from(&bytes, Coordinate { x: 0, y: 0 });
}

mod contig;
mod playfield;
