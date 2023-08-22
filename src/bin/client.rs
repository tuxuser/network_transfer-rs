use network_transfer::{NetworkTransferProtocol, Client};

fn main() {
    let protocol = NetworkTransferProtocol {};
    let results = protocol.discover().expect("No network-transfer activate console found :(");

    let console = results.first().expect("Failed unwrapping console");
    eprintln!("Using console: {console:#?}");

    let client = Client::from(console);
    let metadata = client.get_metadata().expect("Failed fetching metadata");

    let item = metadata.items.first().expect("Failed to get first item");
    eprintln!("Item: {item:#?}");
    let resp = client.download_item(item).expect("Failed downloading");

    println!("{resp:?}");
    let headers: Vec<Option<&str>> = resp
        .headers_names()
        .iter()
        .map(
            |k|
                resp.header(k)
        ).collect();
    eprintln!("{:#?}", headers)
}
