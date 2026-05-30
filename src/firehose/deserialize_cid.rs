use serde::{Deserialize, Deserializer};

pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    let cid_bytes = Option::<serde_bytes::ByteBuf>::deserialize(d)?;
    // CBOR tag 42 prepends a 0x00 multibase identity prefix — strip it
    Ok(cid_bytes.map(|bytes| {
        let raw = match bytes.as_ref() {
            [0x00, rest @ ..] => rest,
            other => other,
        };
        multibase::encode(multibase::Base::Base32Lower, raw)
    }))
}
