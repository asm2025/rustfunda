use crate::{Result, error::RmxError};
use byteorder::{BigEndian, ReadBytesExt};
use std::io::Cursor;

pub trait ReadFromBytes: Sized {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self>;
}

pub fn read_value<T: ReadFromBytes>(bytes: &[u8], offset: &mut usize) -> Result<T> {
    let remaining = &bytes[*offset..];
    let mut cursor = Cursor::new(remaining);
    let value = T::read_from(&mut cursor)?;
    *offset += cursor.position() as usize;
    Ok(value)
}

pub fn read_slice<'a>(bytes: &'a [u8], offset: &mut usize, len: usize) -> Result<&'a [u8]> {
    let end = offset.saturating_add(len);

    if end > bytes.len() {
        return Err(RmxError::Argument(
            "Not enough bytes to read slice".to_string(),
        ));
    }

    if *offset > bytes.len() || len > bytes.len().saturating_sub(*offset) {
        return Err(RmxError::Argument(
            "Slice length would exceed buffer bounds".to_string(),
        ));
    }

    let slice = &bytes[*offset..end];
    *offset = end;
    Ok(slice)
}

// Unsigned integers
impl ReadFromBytes for u8 {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor
            .read_u8()
            .map_err(|_| RmxError::Argument("Failed to read u8".to_string()))
    }
}

impl ReadFromBytes for u16 {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor
            .read_u16::<BigEndian>()
            .map_err(|_| RmxError::Argument("Failed to read u16".to_string()))
    }
}

impl ReadFromBytes for u32 {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor
            .read_u32::<BigEndian>()
            .map_err(|_| RmxError::Argument("Failed to read u32".to_string()))
    }
}

impl ReadFromBytes for u64 {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor
            .read_u64::<BigEndian>()
            .map_err(|_| RmxError::Argument("Failed to read u64".to_string()))
    }
}

impl ReadFromBytes for u128 {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor
            .read_u128::<BigEndian>()
            .map_err(|_| RmxError::Argument("Failed to read u128".to_string()))
    }
}

// Signed integers
impl ReadFromBytes for i8 {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor
            .read_i8()
            .map_err(|_| RmxError::Argument("Failed to read i8".to_string()))
    }
}

impl ReadFromBytes for i16 {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor
            .read_i16::<BigEndian>()
            .map_err(|_| RmxError::Argument("Failed to read i16".to_string()))
    }
}

impl ReadFromBytes for i32 {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor
            .read_i32::<BigEndian>()
            .map_err(|_| RmxError::Argument("Failed to read i32".to_string()))
    }
}

impl ReadFromBytes for i64 {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor
            .read_i64::<BigEndian>()
            .map_err(|_| RmxError::Argument("Failed to read i64".to_string()))
    }
}

impl ReadFromBytes for i128 {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor
            .read_i128::<BigEndian>()
            .map_err(|_| RmxError::Argument("Failed to read i128".to_string()))
    }
}

// Floating point
impl ReadFromBytes for f32 {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor
            .read_f32::<BigEndian>()
            .map_err(|_| RmxError::Argument("Failed to read f32".to_string()))
    }
}

impl ReadFromBytes for f64 {
    fn read_from(cursor: &mut Cursor<&[u8]>) -> Result<Self> {
        cursor
            .read_f64::<BigEndian>()
            .map_err(|_| RmxError::Argument("Failed to read f64".to_string()))
    }
}
