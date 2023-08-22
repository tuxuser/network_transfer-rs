use anyhow::{Result, Context};
use network_transfer::{NetworkTransferProtocol, Client};

fn main() -> Result<()> {
    let protocol = NetworkTransferProtocol {};
    let results = protocol.discover()
        .context("No network-transfer activate console found :(")?;

    let console = results.first()
        .context("Failed unwrapping console")?;
    
    eprintln!("Using console: {console:#?}");

    let client = Client::from(console);
    let metadata = client.get_metadata()
        .context("Failed fetching metadata")?;

    let item = metadata.items.first()
        .context("Failed to get first item")?;

    eprintln!("Item: {item:#?}");
    let resp = client.download_item(item)
        .context("Failed downloading")?;

    println!("{resp:?}");
    let headers: Vec<Option<&str>> = resp
        .headers_names()
        .iter()
        .map(
            |k|
                resp.header(k)
        ).collect();
    eprintln!("{:#?}", headers);

    Ok(())
}
