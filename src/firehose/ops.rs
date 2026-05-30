use crate::firehose::deserialize_cid;
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize, Default)]
pub struct RepoOp {
    pub action: String, // "create" | "update" | "delete"
    pub path: String,   // e.g. "app.bsky.feed.post/3jqyd..."
    #[serde(default, deserialize_with = "deserialize_cid::deserialize")]
    pub cid: Option<String>, // Only for "create" and "update"
}
