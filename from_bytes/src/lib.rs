use std::convert::TryInto;

pub use from_bytes_derive::*;

pub trait FromBytes {
    fn load_from_bytes(&mut self, bytes: &[u8]) -> ReadFromBytesResult<()>;
    fn bytes_size(&self) -> usize;
}

pub type ReadFromBytesResult<T> = Result<T, ReadFromBytesError>;

#[derive(Debug, Clone)]
pub enum ReadFromBytesError {
    BytesArrayTooSmall(usize, usize),
    BytesFormatError(String),
}

// Implementations for primitive number types
macro_rules! derive_frombytes_num {
    ($int_type:tt) => {
        impl FromBytes for $int_type {
            fn load_from_bytes(&mut self, bytes: &[u8]) -> ReadFromBytesResult<()> {
                if bytes.len() < self.bytes_size() {
                    return Err(ReadFromBytesError::BytesArrayTooSmall(
                        self.bytes_size(),
                        bytes.len(),
                    ));
                } else {
                    let (int_bytes, _) = bytes.split_at(std::mem::size_of::<$int_type>());
                    *self = $int_type::from_ne_bytes(int_bytes.try_into().or(Err(
                        ReadFromBytesError::BytesArrayTooSmall(self.bytes_size(), bytes.len()),
                    ))?);
                    return Ok(());
                }
            }

            fn bytes_size(&self) -> usize {
                std::mem::size_of::<$int_type>()
            }
        }
    };
}

derive_frombytes_num!(u8);
derive_frombytes_num!(u16);
derive_frombytes_num!(u32);
derive_frombytes_num!(u64);
derive_frombytes_num!(u128);
derive_frombytes_num!(usize);

derive_frombytes_num!(i8);
derive_frombytes_num!(i16);
derive_frombytes_num!(i32);
derive_frombytes_num!(i64);
derive_frombytes_num!(i128);
derive_frombytes_num!(isize);

derive_frombytes_num!(f32);
derive_frombytes_num!(f64);

// Implementation for byte arrays
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BytesArray {
    pub bytes: Vec<u8>,
}

impl FromBytes for BytesArray {
    fn load_from_bytes(&mut self, bytes: &[u8]) -> ReadFromBytesResult<()> {
        if bytes.len() != self.bytes_size() {
            self.bytes.resize(bytes.len(), 0);
        }
        self.bytes.copy_from_slice(bytes);
        Ok(())
    }

    fn bytes_size(&self) -> usize {
        return self.bytes.capacity();
    }
}

// Implementation for strings
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InlineCString {
    pub contents: String,
}

impl FromBytes for InlineCString {
    fn load_from_bytes(&mut self, bytes: &[u8]) -> ReadFromBytesResult<()> {
        let end_idx = bytes
            .iter()
            .position(|elem| *elem == 0)
            .map(|end_pos| end_pos + 1)
            .unwrap_or(bytes.len());
        let res = std::ffi::CStr::from_bytes_with_nul(&bytes[0..end_idx])
            .map_err(|e| ReadFromBytesError::BytesFormatError(e.to_string()))?;
        self.contents = res
            .to_str()
            .map_err(|e| ReadFromBytesError::BytesFormatError(e.to_string()))?
            .to_string();
        Ok(())
    }

    fn bytes_size(&self) -> usize {
        return self.contents.len();
    }
}

impl From<InlineCString> for String {
    fn from(in_str: InlineCString) -> Self {
        in_str.contents
    }
}
