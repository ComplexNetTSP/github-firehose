use crate::firehose::car::{cid_prefix_len, read_varint};
use crate::firehose::record::Record;
use serde::{Deserialize, Deserializer, de};
use std::io::Cursor;

#[allow(dead_code)]
#[derive(Debug)]
pub struct Blocks(pub Vec<Record>);

impl<'de> Deserialize<'de> for Blocks {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // the blocks field is raw bytes in the CBOR frame
        let bytes = serde_bytes::ByteBuf::deserialize(deserializer)?;
        parse_car(bytes.as_ref())
            .map(Blocks)
            .map_err(de::Error::custom)
    }
}

/// Converts raw CID bytes to a base32lower multibase link string.
/// e.g. &[0x01, 0x71, ...] → "bafyrei..."
// fn cid_to_link(cid_bytes: &[u8]) -> String {
//     multibase::encode(multibase::Base::Base32Lower, cid_bytes)
// }

/// Parses a CAR byte slice into a list of decoded records.
///
/// Skips the CAR header, then iterates over varint-prefixed blocks.
/// Each block has a CID prefix (stripped) followed by DAG-CBOR bytes (decoded).
/// Blocks that don't match a known Record type are silently skipped.
fn parse_car(car_bytes: &[u8]) -> anyhow::Result<Vec<Record>> {
    let mut cursor = Cursor::new(car_bytes);
    let mut records = Vec::new();

    // skip CAR header
    let header_len = read_varint(&mut cursor)? as usize;
    let header_start = cursor.position() as usize;
    cursor.set_position((header_start + header_len) as u64);

    // read blocks until EOF
    loop {
        let block_len = match read_varint(&mut cursor) {
            Ok(0) | Err(_) => break,
            Ok(n) => n as usize,
        };

        let start = cursor.position() as usize;
        let end = start + block_len;
        if end > car_bytes.len() {
            break;
        }

        let block_data = &car_bytes[start..end];

        // skip CID prefix, decode CBOR record
        let cid_len = cid_prefix_len(block_data)?;
        //let cid_bytes = &block_data[..cid_len];
        let cbor = &block_data[cid_len..];

        // encode the raw CID bytes as a base32 link string
        //let cid = cid_to_link(cid_bytes);

        match ciborium::from_reader::<Record, _>(&mut Cursor::new(cbor)) {
            Ok(record) => records.push(record),
            Err(_) => {} // not a record block (e.g. CAR root node), skip
        }

        cursor.set_position(end as u64);
    }

    Ok(records)
}
