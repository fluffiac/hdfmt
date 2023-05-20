use std::{fs::File, io::Write};
use std::io::{self, Read};

use readout::{checked, read_out, Checked, ReadOut};

use sized_vec::SizedVec;
use lazy_idk::LazyIdk;

mod sized_vec;
mod lazy_idk;

#[derive(Checked, Debug)]
pub struct HDReplay {
    pub header: Header,
    pub events: Vec<Event>,
}

impl ReadOut for HDReplay {
    fn read_out(buf: &mut impl Read) -> io::Result<Self> {
        use libflate::zlib::Decoder;

        let header = read_out!(buf => Header)?;

        let mut dec = Vec::new();
        Decoder::new(buf)?.read_to_end(&mut dec)?;
        let buf = &mut dec[..].as_ref();

        let mut events = Vec::new();
        loop {
            let e = match read_out!(buf => Event) {
                Ok(e) => e,
                // Err(e) => break,
                Err(_) => return Err(io::Error::new(io::ErrorKind::Other, "somethin went wrong --")),
            };

            events.push(e);

            if buf.len() == 0 {
                break
            }
        }

        Ok(HDReplay { header, events })
    }
}

#[derive(ReadOut, Debug)]
pub struct Header {
     // hdrpl0x00                    
     unknown1  : [u8; 10],           
     pb_score  : f32,                
     unknown2  : f32,                
     unknown3  : u8,                 
     run_score : f32,                
     player_id : u32,                
     unknown4  : [u8; 16],     
     splits    : SizedVec<u16, f32>,
     unknown5  : SizedVec<u32, u32>,
}

// impl checked, using a macro
checked! {|buf => Header| {
    if [0x68, 0x64, 0x72, 0x70, 0x6C, 0x00] != read_out!(buf => [u8; 6])? {
        return Err(io::Error::from(io::ErrorKind::InvalidData));
    }

    Ok(Header::read_out(buf)?)
}}


#[rustfmt::skip]
#[derive(Checked, ReadOut, Debug)]
pub struct InputData {
    left    : u8,
    right   : u8,
    fwd     : u8,
    back    : u8,
    jump    : u8,
    lmb     : u8,
    rmb     : u8,
    mouse_x : i16,
    mouse_y : i16,
}

type EntityId = u32;

#[derive(Checked, ReadOut, Debug)]
#[repr(u8)]
pub enum SpawnData {
    Dagger(EntityId, [u8; 34])   = 0x01,
    Skull(EntityId, [u8; 49])    = 0x06,
    Centipede {
        id: u32, 
        idk0: u32,
        idk1: u32,
        pos: Position,
    }                            = 0x07,
    SpiderPack {
        id: u32, 
        idk0: u32,
        idk1: u32,
        pos: Position,
    }                            = 0x08,
    Idk0A(EntityId, [u8; 42])    = 0x0A,
    SnakePack {
        id: u32,
        idk0: u32,
        idk1: u32,
        pos: Position,
    }                            = 0x10,
    Spawner {
        id: u32,
        idk0: u32,
        idk1: u32,
        pos: Position,
    }                            = 0x11,
    ScuttlePack {
        id: u32, 
        idk0: u32,
        idk1: u32,
        center: Position,
        pack: [Scuttle; 3]
    }                            = 0x13,
    Lazer(EntityId, [u8; 45])    = 0x1B,
    Idk1C(EntityId, [u8; 33])    = 0x1C,
    Boss {
        id: u32,
        idk0: u32,
        idk1: u32,
        pos: Position,
    }                            = 0x1E,
}

#[derive(Checked, ReadOut, Debug)]
pub struct Scuttle {
    oid: u16,
    pos: Position,
}

#[derive(Checked, ReadOut)]
pub struct Position {
    x: f32,
    y: f32,
    z: f32,
}

use std::fmt::{Debug, Formatter};

impl Debug for Position {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}

#[rustfmt::skip]
#[derive(Checked, ReadOut, Debug)]
#[repr(u8)]
pub enum Event {
    Spawn(SpawnData)          = 0x00,
    Position {
        id: u32,
        pos: Position
    }                         = 0x01, 
    Idk02 {
        id: u32,
        a: f32,
        b: f32,
        c: f32,
    }                         = 0x02,
    Idk03{
        id: u32,
        a: [i16; 9],
    }                         = 0x03,
    Idk04{
        id: u32,
        a: [f32; 3], 
    }                         = 0x04,
    WishPosition {
        id: u32, 
        pos: Position
    }                         = 0x05,
    Idk07{
        id: u32,
        a: i16,
        b: 
        i16,
        c: i16,
    }                         = 0x07,
    Idk08 {
        id: u32,
        id2: u32,
        id3: u32
    }                         = 0x08,
    Idk09(EntityId, [u16; 3], [f32; 6]) = 0x09, 
    Idk0A([u8; 47])           = 0x0A,
    Idk0B([u8; 90])           = 0x0B,
    // why though
    Idk0D                     = 0x0D, 
    // maybe dependant on entity type :O
    State(EntityId, u8)       = 0x0F, 
    Input(InputData)          = 0x11,
    FrameEnd                  = 0x14,
    ReplayEnd                 = 0x15,
    Idk16 {
        idk0: [u8; 12],
        idk1: f32,
        idk2: f32,
        idk3: [u8; 8],
    }                         = 0x16, 
    Idk18(LazyIdk)            = 0x18, // jesus christ
    Idk19 {
        idk0: [u8; 4],
        idk1: f32,
    }                         = 0x19,
}