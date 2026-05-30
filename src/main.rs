mod firehose;

use chrono::{DateTime, Datelike};
use ciborium::from_reader;
use clap::Parser;
use firehose::{CborHeader, CommitFrame};
use futures::StreamExt;
use std::{
    io::{BufWriter, Cursor, Write},
    path::Path,
    time::Duration,
};
use tokio_tungstenite::connect_async;

fn flush_buf(
    buf: &mut Vec<CommitFrame>,
    output_dir: &str,
    year: i32,
    month: i32,
    day: i32,
    last_seq: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    // Ensure output directory exists
    let file_path = format!(
        "{}/year={}/month={}/day={}/{}.ndjson",
        output_dir, year, month, day, last_seq
    );
    if let Some(parent) = Path::new(&file_path).parent() {
        std::fs::create_dir_all(parent)?;
    }
    let file = std::fs::File::create(&file_path)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, buf)?;
    for record in buf.iter() {
        serde_json::to_writer(&mut writer, record)?;
        writer.write_all(b"\n")?;
    }

    eprintln!("Flushing frames in file: {}", file_path);
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

/// bsky-firehose: A Rust client for the Bluesky firehose stream
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Output directory for the flushed frames
    #[arg(short, long, default_value_t = String::from("./output"))]
    output_dir: String,

    /// Maximum number of frames to keep in memory before flushing to disk
    #[arg(short, long, default_value_t = 100000)]
    max_buf_size: usize,

    /// Optional starting sequence number to resume from
    #[arg(short, long)]
    start_seq: Option<u64>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    const INIT_BACKOFF: Duration = Duration::from_secs(5);
    let mut last_seq: Option<u64> = args.start_seq; // Starting sequence number (can be adjusted to resume from a specific point)
    let mut backoff = INIT_BACKOFF;
    let mut buf: Vec<CommitFrame> = Vec::with_capacity(args.max_buf_size);
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
                if buf.len() > args.max_buf_size {
                    //parse the date of le first message in the buffer
                    let first_date: &str = buf[0].time.as_ref();
                    let date = DateTime::parse_from_rfc3339(first_date)?;
                    flush_buf(
                        &mut buf,
                        &args.output_dir,
                        date.year(),
                        date.month() as i32,
                        date.day() as i32,
                        last_seq.unwrap_or(0),
                    )
                    .unwrap_or_else(|e| eprintln!("Error flushing buffer: {}", e));
                }
            }
        }
        //
        // If we reach here, it means the connection was closed or failed.
        // Log the error and wait before retrying.
        //
        eprintln!("Disconnected. Retrying in {:?}...", backoff);
        tokio::time::sleep(backoff).await;
        // Exponential backoff with a maximum cap
        backoff = (backoff * 2).min(Duration::from_secs(60));
    }
}
