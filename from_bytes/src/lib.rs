use std::convert::TryInto;

pub use from_bytes_derive::*;

pub trait FromBytes {
    fn load_from_bytes(&mut self, bytes: &[u8]) -> ReadFromBytesResult<()>;
    fn bytes_size(&self) -> usize;
}

pub type ReadFromBytesResult<T> = Result<T, ReadFromBytesError>;

#[derive(Debug, Clone)]
pub enum ReadFromBytesError {
    BytesArrayTooSmall,
    BytesFormatError(String),
}

// Implementations for primitive number types
macro_rules! derive_frombytes_num {
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
pub struct BytesArray {
    pub bytes: Vec<u8>,
    pub len: usize,
}

impl FromBytes for BytesArray {
    fn load_from_bytes(&mut self, bytes: &[u8]) -> ReadFromBytesResult<()> {
        if bytes.len() != self.bytes_size() {
            return Err(ReadFromBytesError::BytesArrayTooSmall);
        } else {
            if self.bytes.capacity() != self.len {
                self.bytes = Vec::with_capacity(self.len)
            }
            self.bytes.copy_from_slice(bytes);
        }
        Ok(())
    }

    fn bytes_size(&self) -> usize {
        return self.len;
    }
}

// Implementation for strings
pub struct InlineCString {
    pub contents: String,
    pub len: usize,
}

impl FromBytes for InlineCString {
    fn load_from_bytes(&mut self, bytes: &[u8]) -> ReadFromBytesResult<()> {
        let res = std::ffi::CStr::from_bytes_with_nul(bytes)
            .map_err(|e| ReadFromBytesError::BytesFormatError(e.to_string()))?;
        self.contents = res
            .to_str()
            .map_err(|e| ReadFromBytesError::BytesFormatError(e.to_string()))?
            .to_string();
        Ok(())
    }

    fn bytes_size(&self) -> usize {
        return self.len;
    }
}
