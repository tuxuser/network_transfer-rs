use std::net::Ipv4Addr;
use anyhow::{Result, Context};
use network_transfer::{Console, NetworkTransferProtocol};

fn main() -> Result<()> {
    let protocol = NetworkTransferProtocol {};

    let console = Console {
        address: Ipv4Addr::new(10,0,0,229),
        port: 1248,
        id: "X1234".into(),
        name: "XBOXTEST".into(),
    };

    let _ = protocol.announce(&console)
        .context("Failed announcing network-transfer service")?;

    Ok(())
}
