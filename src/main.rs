mod firehose;

use ciborium::from_reader;
use firehose::{CborHeader, CommitFrame, Record};
use futures::StreamExt;
use std::io::Cursor;
use tokio_tungstenite::connect_async;

// Bluesky firehose WebSocket endpoint
const FIREHOSE_URL: &str = "wss://bsky.network/xrpc/com.atproto.sync.subscribeRepos";
fn firehose_url(cursor: Option<u64>) -> String {
    match cursor {
        Some(seq) => format!("{FIREHOSE_URL}?cursor={seq}"),
        None => FIREHOSE_URL.to_string(),
    }
}

fn handle_message(data: Vec<u8>) -> anyhow::Result<()> {
    let mut cursor = Cursor::new(&data);

    // First CBOR item = frame header
    let header: CborHeader = from_reader(&mut cursor)?;

    if header.event_type.as_deref() != Some("#commit") {
        return Ok(());
    }

    let commit: CommitFrame = ciborium::from_reader(&mut cursor)?;

    for record in &commit.blocks.0 {
        match record {
            Record::Post(p) => println!("[{}] POST: {}", commit.seq, p.text),
            Record::Like(l) => {
                println!(
                    "[{}] LIKE -> {}, ops: {:?}",
                    commit.seq, l.subject.uri, commit.ops
                )
            }
            Record::Follow(f) => println!("[{}] FOLLOW -> {}", commit.seq, f.subject),
            Record::Unknown => {}
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() {
    let seq: u64 = 30391757702;
    let url = firehose_url(Some(seq)); // Start from a specific sequence number
    println!("Connecting to firehose at URL: {}", url);
    // Establish the WebSocket connection
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");

    // Split the WebSocket stream into a sender and receiver
    let (_, mut read) = ws_stream.split();

    // Listen for incoming messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(tokio_tungstenite::tungstenite::Message::Binary(data)) => {
                if let Err(e) = handle_message(data) {
                    eprintln!("Failed to decode frame: {}", e);
                }
            }
            Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => {
                println!("Connection closed");
                break;
            }
            Err(e) => eprintln!("Error: {}", e),
            _ => {}
        }
        break; // Remove this break to keep listening indefinitely
    }
}
