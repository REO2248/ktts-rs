use std::fmt;

pub type DataMap = std::collections::HashMap<String, Vec<u8>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DictError {
    pub msg: String,
    pub offset: usize,
}

impl DictError {
    pub fn new(msg: impl Into<String>, offset: usize) -> Self {
        Self {
            msg: msg.into(),
            offset,
        }
    }
}

impl fmt::Display for DictError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "offset 0x{:x}: {}", self.offset, self.msg)
    }
}

impl std::error::Error for DictError {}

pub type DictResult<T> = Result<T, DictError>;

#[derive(Debug)]
pub struct Reader<'a> {
    pub buf: &'a [u8],
    pub pos: usize,
}

impl<'a> Reader<'a> {
    #[must_use]
    pub const fn new(buf: &'a [u8]) -> Self {
        Reader { buf, pos: 0 }
    }
    /// Reads one byte.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is exhausted.
    pub fn u8(&mut self) -> DictResult<u8> {
        let b = *self
            .buf
            .get(self.pos)
            .ok_or_else(|| DictError::new("u8 EOF", self.pos))?;
        self.pos += 1;
        Ok(b)
    }
    /// Reads two bytes in little-endian order.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is exhausted.
    pub fn u16(&mut self) -> DictResult<u16> {
        let v = self
            .buf
            .get(self.pos..self.pos + 2)
            .ok_or_else(|| DictError::new("u16 EOF", self.pos))?;
        self.pos += 2;
        Ok(u16::from_le_bytes([v[0], v[1]]))
    }
    /// Reads four bytes in little-endian order.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is exhausted.
    pub fn u32(&mut self) -> DictResult<u32> {
        let v = self
            .buf
            .get(self.pos..self.pos + 4)
            .ok_or_else(|| DictError::new("u32 EOF", self.pos))?;
        self.pos += 4;
        Ok(u32::from_le_bytes([v[0], v[1], v[2], v[3]]))
    }
    /// Reads a little-endian `f32`.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is exhausted.
    pub fn f32(&mut self) -> DictResult<f32> {
        Ok(f32::from_bits(self.u32()?))
    }
    /// Reads a little-endian `f64`.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is exhausted.
    pub fn f64(&mut self) -> DictResult<f64> {
        let v = self
            .buf
            .get(self.pos..self.pos + 8)
            .ok_or_else(|| DictError::new("f64 EOF", self.pos))?;
        self.pos += 8;
        let mut b = [0u8; 8];
        b.copy_from_slice(v);
        Ok(f64::from_le_bytes(b))
    }
    /// Reads `n` bytes.
    ///
    /// # Errors
    ///
    /// Returns an error if fewer than `n` bytes remain.
    pub fn bytes(&mut self, n: usize) -> DictResult<&'a [u8]> {
        let v = self
            .buf
            .get(self.pos..self.pos + n)
            .ok_or_else(|| DictError::new("bytes EOF", self.pos))?;
        self.pos += n;
        Ok(v)
    }
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.buf.len().saturating_sub(self.pos)
    }
}
