use std::fmt::Debug;
use std::io::{self, Read};

use readout::{read_out, Checked, ReadOut};

#[derive(Checked)]
pub struct LazyIdk(Vec<u8>);

impl ReadOut for LazyIdk {
    fn read_out(buf: &mut impl Read) -> io::Result<Self> {
        let mut vec = Vec::new();
        if let Ok(array) = read_out!(buf => [u8; 13]) {
            vec.extend_from_slice(&array);
        } else {
            buf.read_to_end(&mut vec)?;
            return Ok(Self(vec))
        }

        while let &[a, b, c, d, e, f, g, h, .., l] = &vec[vec.len() - 13..vec.len()] {
            let sum = b as u16 + c as u16 + d as u16 + e as u16 + f as u16 + g as u16 + h as u16;

            if a == 0x11 && l == 0x14 {
                let mut is_complete = true;

                if b != 0x00 && b != 0x01 { is_complete = false }
                if c != 0x00 && c != 0x01 { is_complete = false }
                if d != 0x00 && d != 0x01 { is_complete = false }
                if e != 0x00 && e != 0x01 { is_complete = false }
                if f != 0x00 && f != 0x01 && f != 0x02 { is_complete = false }
                if g != 0x00 && g != 0x01 && g != 0x02 && g != 0x03 { is_complete = false }
                if h != 0x00 && h != 0x01 && h != 0x02 && h != 0x03 { is_complete = false }

                if is_complete {
                    break;
                }
            }

            match read_out!(buf => u8) {
                Ok(byte) => vec.push(byte),
                Err(_) if vec.last().unwrap() == &0x15 => break,
                Err(e) => Err(e)?,
            }
        }

        Ok(LazyIdk(vec))
    }
}

impl LazyIdk {
    pub fn get(&self, i: usize) -> u8 {
        self.0[i]
    }
}

impl Debug for LazyIdk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}
