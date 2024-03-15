use std::{net::IpAddr, io::{Seek, SeekFrom, Read}, os::unix::prelude::FileExt};
use anyhow::{anyhow, Result};
use axum_range::{KnownSize, Ranged};
use network_transfer::{Console, NetworkTransferProtocol};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use axum::{
    extract::{Json, TypedHeader, Path, Extension, Query, FromRequestParts},
    http::{Request, header::{HeaderMap, self, RANGE}, StatusCode, self, request::Parts},
    body::{Bytes, Body, Full},
    headers::{UserAgent, self, Range},
    response::{IntoResponse, Response},
    routing::get,
    Router, async_trait,
};
use serde_json::{Value, json};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
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

    let arr1: [u8; 6] = thread_rng().gen();
    let console_id = format!("X{}", hex::encode(arr1));

    let console = Console {
        address: bind_host,
        port: 10248,
        id: console_id.into(),
        name: "XBOXTEST".into(),
    };

    std::thread::spawn(move || {
        protocol.announce(&console)
            .expect("Failed announcing network-transfer service");
    });

    loop {}
    
    /*
    let app = Router::new()
        .route("/col/metadata", get(get_metadata))
        .route("/col/content/:filename", get(get_content))
        .fallback(fallback_handler);

    // run it with hyper on localhost:3000
    axum::Server::bind(&SocketAddr::new(IpAddr::V4(bind_host), 10248))
        .serve(app.into_make_service())
        .await
        .unwrap();
    */

    Ok(())
}


/*
Handlers
*/
async fn fallback_handler(request: Request<Body>) {
    dbg!(request);
}

async fn get_metadata(headers: HeaderMap) -> impl IntoResponse {
    dbg!(headers);

/*
ORIGINAL
< HTTP/1.1 200 OK
< Content-Type: text/json
< Server: Microsoft-HTTPAPI/2.0
< Date: Sun, 08 Oct 2023 00:33:34 GMT
< Content-Length: 141092

OWN
< HTTP/1.1 200 OK
< content-type: text/json
< server: Microsoft-HTTPAPI/2.0
< content-length: 469
< date: Sun, 08 Oct 2023 00:35:53 GMT

*/

    let body = Json(json!(
        {"items":[{
            "type": "app",
            "hasContentId": false,
            "contentId": "",
            "productId": "",
            "packageFamilyName": "11032Reconco.XboxControllerTester_thvmwcgtjwwvy",
            "oneStoreProductId": "9NBLGGH4PNC7",
            "version": "281505043185734",
            "size": 105205760,
            "allowedProductId": "",
            "allowedPackageFamilyName": "",
            "path": "/col/content/%7BA89ECE52-7E8E-444F-BBD0-C68B76C2ECA4%7D%2311032Reconco.XboxControllerTester_thvmwcgtjwwvy",
            "availability": "available",
            "generation": "uwpgen9",
            "relatedMedia": [],
            "relatedMediaFamilyNames": []
        }]}
    ));

    (
        [
            ("Content-type", "text/json"),
            ("Server", "Microsoft-HTTPAPI/2.0")
        ],
        body
    )
}

async fn get_content(Path(filename): Path<String>, range: Option<TypedHeader<Range>>) -> impl IntoResponse
{
    dbg!(&filename, &range);

    let (drive_id, filename) = {
        let mut pair = filename.split("#");
        let drive_id = uuid::Uuid::parse_str(pair.next().unwrap()).unwrap();
        (drive_id, pair.next().unwrap().to_owned())
    };

    let file = tokio::fs::File::open(&filename).await.unwrap();
    let body = KnownSize::file(file).await.unwrap();
    let range = range.map(|TypedHeader(range)| range);
    (
        [
            ("Content-Type","application/octet-stream"),
            ("Server","Microsoft-HTTPAPI/2.0")
        ],
        Ranged::new(range, body)
    )
    
/*
ORIGINAL

< HTTP/1.1 206 OK
< Content-Type: application/octet-stream
< Content-Range: bytes 0-0/105205760
< Server: Microsoft-HTTPAPI/2.0
< Date: Sun, 08 Oct 2023 00:18:01 GMT
< Content-Length: 1

OUR

< HTTP/1.1 206 Partial Content
< content-range: bytes 0-0/5242880
< content-length: 1
< accept-ranges: bytes
< content-type: application/octet-stream
< server: Microsoft-HTTPAPI/2.0
< date: Sun, 08 Oct 2023 00:27:08 GMT

*/
}