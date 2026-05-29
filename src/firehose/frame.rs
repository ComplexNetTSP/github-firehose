use crate::firehose::blocks::Blocks;
use crate::firehose::ops::RepoOp;
use serde::Deserialize;
/// Event type:
///    - "#commit": Applying changes to a user's repository.
///    - "#handle": Handling a user's repository.
///    - "#identity": Updating a user's DID document.
///    - "#tombstone": Soft-deleting a post.
///    - "#account": Creating or updating a user's account.
///    - "#sync": Emitted after a period of inactivity, or when a consumer falls behind and misses too many events. Contains a full snapshot of the current repo state, so consumers can sync back up without needing to fetch historical commits.
///    - "#info": Emitted periodically with information about the firehose stream,
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CborHeader {
    #[serde(rename = "t")]
    pub event_type: Option<String>,
    pub op: i64, // 0 = Request, 1 = Response, 2 = Error, 3 = Stream Frame
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct CommitFrame {
    //pub blobs: Vec<u8>,

    // blocks: CAR file containing relevant blocks, as a diff since the previous
    // repo state. The commit must be included as a block,
    // and the commit block CID must be the first entry in the CAR header 'roots' list.
    pub blocks: Blocks,
    pub ops: Vec<RepoOp>,

    // prevData: The root CID of the MST tree for the previous commit from this repo
    // (indicated by the 'since' revision field in this message).
    // Corresponds to the 'data' field in the repo commit object.
    // NOTE: this field is effectively required for the 'inductive' version of firehose.
    #[serde(rename = "prevData")]
    pub prev_data: Option<serde_bytes::ByteBuf>,

    // rebase: DEPRECATED -- unused
    pub rebase: bool,

    // repo: The repo this event comes from.
    // Note that all other message types name this field 'did'
    pub repo: String,

    // rev: The rev of the emitted commit.
    // Note that this information is also in the commit object included in blocks,
    // unless this is a tooBig event.
    pub rev: String,

    // seq: The stream sequence number of this message.
    pub seq: u64,

    // since: The rev of the last emitted commit from this repo (if any).
    pub since: String,

    // time: Timestamp of when this message was originally broadcast.
    // ISO 8601 timestamp
    pub time: String,

    // tooBig: DEPRECATED -- replaced by #sync event and data limits.
    // Indicates that this commit contained too many ops, or data size was too large.
    // Consumers will need to make a separate request to get missing data.
    #[serde(rename = "tooBig")]
    pub too_big: bool,
}
