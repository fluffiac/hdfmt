use std::io::Read;

///////////////////////////////////////////////////////////////////////////////
// traits

pub trait ReadOut: Sized {
    fn read_out(buf: &mut impl Read) -> std::io::Result<Self>;
}

pub trait Checked: ReadOut {
    #[inline(always)]
    fn check(buf: &mut impl Read) -> std::io::Result<Self> {
        Self::read_out(buf)
    }
}

///////////////////////////////////////////////////////////////////////////////
// macros

/// derive macros
pub use readout_derive::{Checked, ReadOut};

/// macro for data checking boilerplate
#[macro_export]
macro_rules! checked {
    (|$buf:ident => $t:ty| $bl:block) => {
        impl $crate::Checked for $t {
            fn check($buf: &mut impl std::io::Read) -> std::io::Result<Self> {
                $bl
            }
        }
    };
}

/// macro for read_out boilerplate
#[macro_export]
macro_rules! read_out {
    ($buffer:expr => $t:ty) => {
        <$t as $crate::Checked>::check($buffer)
    };
}

///////////////////////////////////////////////////////////////////////////////
// impl primitives

macro_rules! primitive_impl {
    ($($t:ty),*) => {
        $(
            impl $crate::Checked for $t {}
            impl $crate::ReadOut for $t {
                fn read_out(buf: &mut impl Read) -> std::io::Result<Self> {
                    let mut out = [0; std::mem::size_of::<Self>()];
                    buf.read_exact(&mut out)?;
                    Ok(Self::from_le_bytes(out))
                }
            }
        )*
    };
}

primitive_impl!(u8, u16, u32, usize, u64, u128, i8, i16, i32, isize, i64, i128, f32, f64);

impl<T: Checked, const C: usize> Checked for [T; C] {}
impl<T: Checked, const C: usize> ReadOut for [T; C] {
    fn read_out(buffer: &mut impl Read) -> std::io::Result<Self> {
        use std::mem::MaybeUninit;

        let mut out: [MaybeUninit<T>; C] = unsafe { MaybeUninit::uninit().assume_init() };

        for i in 0..C {
            out[i].write(read_out!(buffer => T)?);
        }

        unsafe { Ok(std::mem::transmute_copy(&out)) }        
    }
}

impl Checked for () {}
impl ReadOut for () {
    fn read_out(buffer: &mut impl Read) -> std::io::Result<Self> {
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////
// tests

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Checked, ReadOut, PartialEq, Eq)]
    #[repr(u8)]
    enum DerivedEnum {
        A(u8, u8) = 0x00,
        B { c: u8, d: u8 } = 0x01,
        C = 0x02,
    }

    #[test]
    fn test_read_out() {
        let data = [0xde, 0xad, 0xbe, 0xef];

        let buf = &mut data.as_ref();
        assert!(read_out!(buf => u32).unwrap() == 0xefbeadde);
    }

    #[test]
    fn test_struct_derive() {
        #[derive(Checked, ReadOut)]
        struct TestUnit;

        #[derive(Checked, ReadOut, PartialEq, Eq)]
        struct Test {
            a: u32,
            b: u32,
        }

        let data = [0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef];
        let buf = &mut data.as_ref();

        assert!(
            read_out!(buf => Test).unwrap()
                == Test {
                    a: 0xefbeadde,
                    b: 0xefbeadde
                }
        );
    }

    #[test]
    fn test_checked() {
        #[derive(ReadOut, PartialEq, Eq)]
        struct Test {
            a: u16,
            b: u32,
        }

        checked! {|buf => Test| {
            if [0xde, 0xad] == read_out!(buf => [u8; 2])? {
                Ok(ReadOut::read_out(buf)?)
            } else {
                Err(std::io::Error::from(std::io::ErrorKind::InvalidData))
            }
        }}

        let data = [0xde, 0xad, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef];
        let buf = &mut data.as_ref();

        assert!(
            read_out!(buf => Test).unwrap()
                == Test {
                    a: 0xefbe,
                    b: 0xefbeadde
                }
        );

        let mangled = [0x00, 0x00, 0xbe, 0xef, 0xde, 0xad, 0xbe, 0xef];
        let buf = &mut mangled.as_ref();

        assert!(read_out!(buf => Test).is_err());
    }

    #[test]
    fn enum_derive() -> std::io::Result<()> {
        use DerivedEnum::*;

        let data = [0x00, 0xde, 0xad, 0x01, 0xbe, 0xef, 0x2];
        let buf = &mut data.as_ref();

        assert!(read_out!(buf => DerivedEnum)? == A(0xde, 0xad));
        assert!(read_out!(buf => DerivedEnum)? == B { c: 0xbe, d: 0xef });
        assert!(read_out!(buf => DerivedEnum)? == C);

        Ok(())
    }

    #[test]
    #[should_panic]
    fn mangled_enum() {
        let data = [0x03, 0xde, 0xad];
        let buf = &mut data.as_ref();

        read_out!(buf => DerivedEnum).unwrap();
    }
}
