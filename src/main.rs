mod firehose;

use ciborium::from_reader;
use firehose::{CborHeader, CommitFrame};
use futures::StreamExt;
use serde_json;
use std::io::{BufWriter, Write};
use std::{io::Cursor, time::Duration};
use tokio_tungstenite::connect_async;

fn flush_buf(buf: &mut Vec<CommitFrame>) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create("data.ndjson")?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, buf)?;
    for record in buf.iter() {
        serde_json::to_writer(&mut writer, record)?;
        writer.write_all(b"\n")?;
    }

    println!("Flushing {} frames", buf.len());
    buf.clear();
    Ok(())
}

fn firehose_url(cursor: Option<u64>) -> String {
    // Bluesky firehose WebSocket endpoint
    const FIREHOSE_URL: &str = "wss://bsky.network/xrpc/com.atproto.sync.subscribeRepos";
    match cursor {
        Some(seq) => format!("{FIREHOSE_URL}?cursor={seq}"),
        None => FIREHOSE_URL.to_string(),
    }
}

fn handle_message(data: Vec<u8>, buf: &mut Vec<CommitFrame>) -> anyhow::Result<Option<u64>> {
    let mut cursor = Cursor::new(&data);

    // First CBOR item = frame header
    let header: CborHeader = from_reader(&mut cursor)?;
    // Only process commit frames for now
    if header.event_type.as_deref() != Some("#commit") {
        return Ok(None);
    }
    let commit: CommitFrame = ciborium::from_reader(&mut cursor)?;
    let seq = commit.seq;
    buf.push(commit);
    Ok(Some(seq))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    const INIT_BACKOFF: Duration = Duration::from_secs(5);
    const MAX_BUF_SIZE: usize = 100; // Max number of frames to keep in memory
    let mut last_seq: Option<u64> = None; // Starting sequence number (can be adjusted to resume from a specific point)
    let mut backoff = INIT_BACKOFF;
    let mut buf: Vec<CommitFrame> = Vec::with_capacity(MAX_BUF_SIZE);
    // Continuously attempt to connect and listen to the firehose, with a backoff strategy on failure
    loop {
        let url = firehose_url(last_seq); // Start from a specific sequence number
        println!("Connecting to firehose at URL: {}", url);
        // Establish the WebSocket connection
        if let Ok((ws_stream, _)) = connect_async(url).await {
            // Reset backoff on successful connection
            backoff = INIT_BACKOFF;

            // Split the WebSocket stream into a sender and receiver
            let (_, mut read) = ws_stream.split();

            // Listen for incoming messages
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(tokio_tungstenite::tungstenite::Message::Binary(data)) => {
                        if let Ok(Some(seq)) = handle_message(data, &mut buf) {
                            last_seq = Some(seq);
                        }
                    }
                    Ok(tokio_tungstenite::tungstenite::Message::Close(_)) => break,
                    _ => break, // Ignore other message types and errors for simplicity
                }
                // If buffer exceeds max size, remove the oldest frame
                if buf.len() > MAX_BUF_SIZE {
                    flush_buf(&mut buf)
                        .unwrap_or_else(|e| eprintln!("Error flushing buffer: {}", e));
                }
            }
        }
        //
        // If we reach here, it means the connection was closed or failed.
        // Log the error and wait before retrying.
        //
        println!("Disconnected. Retrying in {:?}...", backoff);
        tokio::time::sleep(backoff).await;
        // Exponential backoff with a maximum cap
        backoff = (backoff * 2).min(Duration::from_secs(60));
    }
}
