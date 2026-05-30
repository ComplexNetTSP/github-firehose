# Streaming and Decoding Bluesky Firehose 

## The Full Journey: Raw WebSocket Message → Readable Post Text

---

### 1. A binary WebSocket message arrives

After a websocket Connecte to the [Bluesky Firehose endpoint]("wss://relay1.us-east.bsky.network/xrpc/com.atproto.sync.subscribeRepos"), you receive a CBOR frame (binary object). Bluesky uses a variant called [DAG-CBOR](https://ipld.io/specs/codecs/dag-cbor/spec/).

---

### 2. Decode the first CBOR object — the frame header

This tells you what kind of message it is. It looks like `{ op: 1, t: "#commit" }`. The `op` field says whether it's a real event (`1`) or an error (`-1`). The `t` field says the event type — you mostly care about `#commit`, which means someone created, updated, or deleted a record.

---

### 3. Decode the second CBOR object — the frame body

This is the actual event payload. It contains:

- `repo` — the DID of the user whose repo changed (e.g. `did:plc:abc123`)
- `seq` — a sequence number, useful for resuming
- `ops` — a list of changes that happened (each one has an action, a path, and a CID pointer)
- `blocks` — a blob of raw bytes containing the actual record data (this is the CAR)

---

### 4. Look at the `ops` list

Each op describes one change. The `path` tells you what kind of record it is — e.g. `app.bsky.feed.post/3jqyd...` means a post was created. The `action` is `create`, `update`, or `delete`. The `cid` is a hash that acts like a pointer into the `blocks` blob, telling you exactly which block contains this record's data.

---

### 5. Parse the CAR (the `blocks` blob)

CAR is just a packaging format. Think of it like a tiny zip file. It starts with a small header, then a sequence of blocks. Each block is prefixed by its CID (the hash/identifier), followed by the actual record bytes. You read through it and build a lookup table of `CID → record bytes`.

---

### 6. Match each op's CID to a block

Take the CID from the op, find the matching entry in your lookup table, and you now have the raw bytes for that specific record.

---

### 7. Decode the block — it's DAG-CBOR

Deserialize those bytes as CBOR. You get a map with a `$type` field that tells you the Lexicon type, plus all the record fields. For a post that's `app.bsky.feed.post` and the `text` field is the post content. For a like it's `app.bsky.feed.like`. And so on.

---

## The Short Version

> WebSocket frame → split into two CBORs → header tells you the type → body gives you ops + a block store → each op has a CID pointing into the block store → look up the block → decode it as CBOR → read the fields

---

## The Tricky Parts

| # | Issue | Detail |
|---|-------|--------|
| 1 | Double-CBOR framing | Most CBOR libraries decode one item at a time; you need to decode twice from a cursor to get both the header and body |
| 2 | CAR varint parsing | The CAR format uses unsigned varints to length-prefix each block; there's no standard Rust crate that handles this end-to-end cleanly |
| 3 | CID matching | The op's CID (from CBOR tag 42) has a leading `0x00` multibase identity prefix that must be stripped before comparing to the CID parsed from the CAR block |
