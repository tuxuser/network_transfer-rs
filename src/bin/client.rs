use std::io::Seek;

use anyhow::{Result, Context};
use env_logger::Env;
use network_transfer::{NetworkTransferProtocol, Client};

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

    let size = client.download_item(item, &mut file)
        .context("Failed downloading")?;

    assert_eq!(size, file.stream_position()? as usize);

    Ok(())
}
