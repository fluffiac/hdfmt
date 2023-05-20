use std::io::Read;
use std::fs::{File};
use std::path::{PathBuf, Path};
use std::io;

mod hdreplay;

pub use hdreplay::HDReplay;

///////////////////////////////////////////////////////////////////////////////
// some utils

pub fn buffer_from_file<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    File::open(path)?.read_to_end(&mut buf)?;
    Ok(buf)
}

pub fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<impl Iterator<Item = PathBuf>> {
    let dir = std::fs::read_dir(path)?;
    
    Ok(dir.map(|r| r.map(|de| de.path())).flatten())
}