use std::io;

use readout::read_out;

use hdfmt::{buffer_from_file, HDReplay};

fn main() -> io::Result<()> {
    let hdr_buf = buffer_from_file("./dat/decide.hdreplay")?;

    let replay = read_out!(&mut hdr_buf[..].as_ref() => HDReplay)?;

    for event in replay.events {
        println!("{event:?}");
    }

    Ok(())
}