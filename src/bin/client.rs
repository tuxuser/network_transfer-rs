use std::io::Seek;

use anyhow::{Result, Context};
use env_logger::Env;
use indicatif::{ProgressBar, ProgressStyle};
use network_transfer::{models::MetadataItem, Client, NetworkTransferProtocol};

const STEP_SIZE: usize = 0x10000;

fn download_with_progress(client: &Client, item: &MetadataItem, writer: &mut (impl std::io::Write + std::io::Seek)) -> Result<usize> {
    let content_length = client.get_item_filesize(item)?;
    dbg!(content_length);

    let progress_style = ProgressStyle::with_template("[{elapsed_precise}] [ETA: {eta}] {bar:40.cyan/blue} {bytes:>7}/{total_bytes:7} ({bytes_per_sec}) {msg}")?;
    let progress = ProgressBar::new(content_length as u64)
        .with_style(progress_style);

    let mut buf = vec![0u8; STEP_SIZE];
    let written: usize = Client::iterate_range(content_length, STEP_SIZE).map(move |range|{
        let resp = client.download_chunk(&item.path, &range).unwrap();
        resp.into_reader().read_exact(&mut buf[..range.count()]).unwrap();
        let wrote = writer.write(&buf[..range.count()]).unwrap();
        progress.inc(wrote as u64);

        wrote
    })
    .sum();

    Ok(written)
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let protocol = NetworkTransferProtocol {};
    let results = protocol.discover()
        .context("No network-transfer activate console found :(")?;

    let console = results.first()
        .context("Failed unwrapping console")?;
    
    log::info!("Using console: {console:#?}");

    let client = Client::from(console);
    let metadata = client.get_metadata()
        .context("Failed fetching metadata")?;

    let item = metadata.items.first()
        .context("Failed to get first item")?;

    log::info!("Item: {item:#?}");

    let mut file = std::fs::File::create(&item.package_family_name)?;

    let size = download_with_progress(&client, item, &mut file)
        .context("Failed downloading")?;

    assert_eq!(size, file.stream_position()? as usize);

    Ok(())
}
