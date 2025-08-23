use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::io::{Cursor, Read};
use util::{Result, error::RmxError};

pub const DATA_COLLECTION_ADDRESS: &str = "127.0.0.1:9004";

const VERSION_NUMBER: u16 = 1;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Metrics {
    pub total_memory: u64,
    pub used_memory: u64,
    pub cpus: usize,
    pub cpu_usage: f32,     // percent 0.0..100.0
    pub avg_cpu_usage: f32, // average across CPUs
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CollectorCommand {
    SubmitData {
        collector_id: u128,
        metrics: Metrics,
    },
}

pub fn new_collector_id() -> u128 {
    rand::rng().random()
}

pub fn encode(collector_id: u128, command: CollectorCommand) -> Vec<u8> {
    let json = serde_json::to_string(&command).unwrap();
    let bytes = json.as_bytes();
    let crc = crc32fast::hash(bytes);
    let size = bytes.len() as u32;
    let timestamp = util::datetime::unix::now();

    let capacity = size_of::<u128>() // collector_id
        + size_of::<u16>() // VERSION_NUMBER
        + size_of::<u64>() // timestamp
        + size_of::<u32>() // payload size
        + bytes.len() // payload bytes
        + size_of::<u32>(); // CRC

    let mut result = Vec::with_capacity(capacity);

    result.write_u128::<BigEndian>(collector_id).unwrap();
    result.write_u16::<BigEndian>(VERSION_NUMBER).unwrap();
    result.write_u64::<BigEndian>(timestamp).unwrap();
    result.write_u32::<BigEndian>(size).unwrap();
    result.extend_from_slice(bytes);
    result.write_u32::<BigEndian>(crc).unwrap();
    result
}

pub fn decode(collector_id: u128, bytes: &[u8]) -> Result<(u64, CollectorCommand)> {
    let mut cursor = Cursor::new(bytes);

    let magic_number = cursor.read_u128::<BigEndian>()?;
    let version = cursor.read_u16::<BigEndian>()?;
    let timestamp = cursor.read_u64::<BigEndian>()?;
    let size = cursor.read_u32::<BigEndian>()? as usize;
    let mut payload = vec![0u8; size];
    cursor.read_exact(&mut payload)?;
    let crc = cursor.read_u32::<BigEndian>()?;

    if magic_number != collector_id {
        return Err(RmxError::Invalid("Invalid collector id.".to_string()));
    }

    if version != VERSION_NUMBER {
        return Err(RmxError::Invalid("Invalid version number.".to_string()));
    }

    let computed_crc = crc32fast::hash(&payload);
    assert_eq!(crc, computed_crc);

    if crc != computed_crc {
        return Err(RmxError::Invalid("Bad CRC checksum.".to_string()));
    }

    let command = serde_json::from_slice(&payload).unwrap();
    Ok((timestamp, command))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_and_decode() {
        let collector_id = new_collector_id();
        let metrics = Metrics {
            total_memory: 100,
            used_memory: 50,
            cpus: 4,
            cpu_usage: 15.0,
            avg_cpu_usage: 1.5,
        };
        let command = CollectorCommand::SubmitData {
            collector_id,
            metrics,
        };
        let encoded = encode(collector_id, command.clone());
        let (timestamp, decoded) = decode(collector_id, &encoded).unwrap();
        assert!(timestamp > 0);
        assert_eq!(command, decoded);
    }
}
