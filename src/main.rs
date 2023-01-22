use contig::Coordinate;
use playfield::Playfield;

fn main() {
    println!("Hello, world!");

    let bytes0: Vec<Vec<u8>> = vec![vec![0x01, 0x02], vec![0x13, 0x14], vec![0x25, 0x26]];
    let origin0 = Coordinate { x: 0, y: 0 };

    let p = Playfield::<i32, u16>::from(&bytes0, origin0);

    let (bytes1, origin1) = p.to_rows_of_bytes();

    println!("{:?}", bytes1);
    println!("{:?}", origin1);

    // TODO make this a test, it's not working
}

mod contig;
mod playfield;
