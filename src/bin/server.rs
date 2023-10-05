use std::net::IpAddr;
use anyhow::{anyhow, Result};
use network_transfer::{Console, NetworkTransferProtocol};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};

fn main() -> Result<()> {
    let protocol = NetworkTransferProtocol {};

    let network_interfaces = NetworkInterface::show().unwrap();

    let network_interfaces: Vec<NetworkInterface> = network_interfaces
        .into_iter()
        .filter(|intf|
            intf.addr.iter().any(|&addr|{ addr.ip().is_ipv4() && !addr.ip().is_loopback()})
        )
        .collect();
    
    if network_interfaces.is_empty() {
        return Err(anyhow!("No network interfaces enumerated, exiting"));
    }

    let bind_host = loop {
        for (idx, intf) in network_interfaces.iter().enumerate() {
            println!("{idx}) {} ({:?})", intf.name, intf.addr)
        }
    
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input) 
            .expect("Failed to read line");
        let choice: usize = input
            .trim()
            .parse()
            .expect("Input not an integer");
    
        if choice > (network_interfaces.len() - 1) {
            eprintln!("Invalid choice: {choice}, maximum interfaces: {}", network_interfaces.len() - 1);
            continue;
        }

        let bind_result = network_interfaces.get(choice).unwrap().addr.iter().find_map(|addr| {
            match addr.ip() {
                IpAddr::V4(ip4_addr) => Some(ip4_addr),
                _ => None,
            }
        });
    
        break bind_result.unwrap();
    };

    println!("Binding server to host: {bind_host:?}");

    let console = Console {
        address: bind_host,
        port: 1248,
        id: "X1234".into(),
        name: "XBOXTEST".into(),
    };

    std::thread::spawn(move || {
        protocol.announce(&console)
            .expect("Failed announcing network-transfer service");
    });

    Ok(())
}
