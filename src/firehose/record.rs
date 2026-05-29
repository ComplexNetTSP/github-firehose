use serde::Deserialize;

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
#[serde(tag = "$type")]
pub enum Record {
    #[serde(rename = "app.bsky.feed.post")]
    Post(Post),

    #[serde(rename = "app.bsky.feed.like")]
    Like(Like),

    #[serde(rename = "app.bsky.graph.follow")]
    Follow(Follow),

    Unknown, // catches everything else — no data
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Post {
    pub text: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub langs: Option<Vec<String>>,
    pub reply: Option<ReplyRef>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Like {
    pub subject: StrongRef,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct Follow {
    pub subject: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ReplyRef {
    pub root: StrongRef,
    pub parent: StrongRef,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct StrongRef {
    pub uri: String,
    // cid is a CBOR tag 42 — skip it or handle separately
    pub cid: serde_bytes::ByteBuf,
}
