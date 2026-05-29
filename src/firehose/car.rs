/// Low-level CAR format primitives.
///
/// CAR (Content Addressable aRchive) is a packaging format used by Bluesky
/// to bundle repo blocks in firehose commit events. Each block is prefixed
/// by a varint length and a CID.

/// Reads an unsigned variable-length integer (varint) from a cursor.
///
/// Layout: each byte contributes 7 bits. High bit = more bytes follow.
///   0x2A        → 42
///   0x8A 0x01   → 138
pub fn read_varint(cursor: &mut std::io::Cursor<&[u8]>) -> anyhow::Result<u64> {
    use std::io::Read;
    let mut result = 0u64;
    let mut shift = 0;
    loop {
        let mut byte = [0u8; 1];
        cursor.read_exact(&mut byte)?;
        result |= ((byte[0] & 0x7f) as u64) << shift;
        if byte[0] & 0x80 == 0 {
            return Ok(result);
        }
        shift += 7;
    }
}

/// Decodes a varint from a plain byte slice.
/// Returns `(value, bytes_consumed)`.
pub fn decode_varint(data: &[u8]) -> anyhow::Result<(u64, usize)> {
    let mut result = 0u64;
    let mut shift = 0;
    for (i, &byte) in data.iter().enumerate() {
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            return Ok((result, i + 1));
        }
        shift += 7;
    }
    anyhow::bail!("varint not terminated — ran out of bytes")
}

/// Returns how many bytes the CIDv1 prefix occupies at the start of a block.
///
/// CIDv1 layout:
///   [version: varint] [codec: varint] [hash-fn: varint] [digest-len: varint] [digest: bytes]
///
/// Typical SHA2-256 DAG-CBOR CID = 36 bytes:
///   0x01 0x71 0x12 0x20 [32 bytes]
pub fn cid_prefix_len(data: &[u8]) -> anyhow::Result<usize> {
    let mut pos = 0;
    let (_, n) = decode_varint(&data[pos..])?;
    pos += n; // version
    let (_, n) = decode_varint(&data[pos..])?;
    pos += n; // codec
    let (_, n) = decode_varint(&data[pos..])?;
    pos += n; // hash-fn
    let (digest_len, n) = decode_varint(&data[pos..])?;
    pos += n; // digest-len
    pos += digest_len as usize;
    Ok(pos)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn make_cid_v1() -> Vec<u8> {
        let mut cid = vec![0x01, 0x71, 0x12, 0x20];
        cid.extend_from_slice(&[0u8; 32]);
        cid
    }

    #[test]
    fn test_decode_varint_single_byte() {
        let (val, consumed) = decode_varint(&[0x2A]).unwrap();
        assert_eq!(val, 42);
        assert_eq!(consumed, 1);
    }

    #[test]
    fn test_decode_varint_two_bytes() {
        let (val, consumed) = decode_varint(&[0x8A, 0x01]).unwrap();
        assert_eq!(val, 138);
        assert_eq!(consumed, 2);
    }

    #[test]
    fn test_decode_varint_unterminated_returns_error() {
        assert!(decode_varint(&[0x80, 0x80, 0x80]).is_err());
    }

    #[test]
    fn test_read_varint_advances_cursor() {
        let data = [0x2A, 0x01];
        let mut cursor = Cursor::new(data.as_ref());
        assert_eq!(read_varint(&mut cursor).unwrap(), 42);
        assert_eq!(read_varint(&mut cursor).unwrap(), 1);
    }

    #[test]
    fn test_cid_prefix_len_typical() {
        assert_eq!(cid_prefix_len(&make_cid_v1()).unwrap(), 36);
    }

    #[test]
    fn test_cid_prefix_len_points_to_cbor_start() {
        let cbor = b"\xA1\x64text\x65hello";
        let mut data = make_cid_v1();
        data.extend_from_slice(cbor);
        let len = cid_prefix_len(&data).unwrap();
        assert_eq!(&data[len..], cbor.as_ref());
    }
}
