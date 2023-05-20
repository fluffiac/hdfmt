use std::fmt::Debug;
use std::io::{self, Read};

use readout::{read_out, Checked, ReadOut};

pub struct SizedVec<L: Into<u128> + Checked, T: Checked>(Vec<T>, std::marker::PhantomData<L>);

impl<L: Into<u128> + Checked, T: Checked> Checked for SizedVec<L, T> {}
impl<L: Into<u128> + Checked, T: Checked> ReadOut for SizedVec<L, T> {
    fn read_out(buf: &mut impl Read) -> io::Result<Self> {
        let len = read_out!(buf => L)?.into();
        let mut out = Vec::with_capacity(len as usize);
        for _ in 0..len.into() {
            out.push(read_out!(buf => T)?);
        }
        Ok(SizedVec(out, std::marker::PhantomData))
    }
}

impl<L: Into<u128> + Checked, T: Checked + Debug> Debug for SizedVec<L, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.0.iter()).finish()
    }
}
