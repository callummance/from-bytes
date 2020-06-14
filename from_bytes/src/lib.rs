use std::convert::TryInto;

pub use from_bytes_derive::*;

pub trait FromBytes {
    fn load_from_bytes(&mut self, bytes: &[u8]) -> ReadFromBytesResult<()>;
    fn bytes_size(&self) -> usize;
}

macro_rules! derive_frombytes_int {
    ($int_type:tt) => {
        impl FromBytes for $int_type {
            fn load_from_bytes(&mut self, bytes: &[u8]) -> ReadFromBytesResult<()> {
                if bytes.len() < self.bytes_size() {
                    return Err(ReadFromBytesError::BytesArrayTooSmall);
                } else {
                    let (int_bytes, _) = bytes.split_at(std::mem::size_of::<$int_type>());
                    *self = $int_type::from_ne_bytes(
                        int_bytes
                            .try_into()
                            .or(Err(ReadFromBytesError::BytesArrayTooSmall))?,
                    );
                    return Ok(());
                }
            }

            fn bytes_size(&self) -> usize {
                std::mem::size_of::<$int_type>()
            }
        }
    };
}

derive_frombytes_int!(u8);
derive_frombytes_int!(u16);
derive_frombytes_int!(u32);
derive_frombytes_int!(u64);
derive_frombytes_int!(usize);
derive_frombytes_int!(i8);
derive_frombytes_int!(i16);
derive_frombytes_int!(i32);
derive_frombytes_int!(i64);

pub type ReadFromBytesResult<T> = Result<T, ReadFromBytesError>;
#[derive(Debug, Clone)]
pub enum ReadFromBytesError {
    BytesArrayTooSmall,
    BytesFormatError(String),
}
