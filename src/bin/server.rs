use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use anyhow::{anyhow, Result};
use axum_range::{KnownSize, Ranged};
use env_logger::Env;
use network_transfer::{generate_random_console_id, models::{Metadata, MetadataItem}, Console, NetworkTransferProtocol};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use axum::{
    body::Body, extract::{Json, Path, TypedHeader}, headers::Range, http::{header::HeaderMap, Request}, response::IntoResponse, routing::get, Router
};
use serde_json::json;
use network_transfer::{error::Error, SERVER_PORT};

fn get_network_interfaces() -> Result<Vec<NetworkInterface>> {
    let interfaces: Vec<NetworkInterface> = NetworkInterface::show()?
        .into_iter()
        .filter(|intf|
            intf.addr.iter().any(|&addr|{ addr.ip().is_ipv4() && !addr.ip().is_loopback()})
        )
        .collect();

    Ok(interfaces)
}

fn choose_bind_addr(interfaces: &[NetworkInterface]) -> Result<Ipv4Addr> {
    for (idx, intf) in interfaces.iter().enumerate() {
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

    if choice > (interfaces.len() - 1) {
        log::error!("Invalid choice: {choice}, maximum interfaces: {}", interfaces.len() - 1);
        return Err(Error::GeneralError("()".into()).into());
    }

    let bind_result = interfaces
        .get(choice)
        .ok_or(Error::GeneralError("Invalid choice of network interface".into()))?
        .addr
        .iter()
        .find_map(|addr| {
            match addr.ip() {
                IpAddr::V4(ip4_addr) => Some(ip4_addr),
                _ => None,
            }
        })
        .ok_or(Error::GeneralError("Failed to enumerate IPv4Address for choice".into()))?;

    Ok(bind_result)
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let protocol = NetworkTransferProtocol {};

    let network_interfaces = get_network_interfaces()?;
    if network_interfaces.is_empty() {
        return Err(anyhow!("No network interfaces enumerated, exiting"));
    }

    let bind_addr = loop {
        if let Ok(addr) = choose_bind_addr(&network_interfaces) {
            break addr;
        }
    };

    log::info!("Binding server to host: {bind_addr:?}");

    let console_id = generate_random_console_id();

    let console = Console {
        address: bind_addr,
        port: SERVER_PORT,
        id: console_id,
        name: "XBOXTEST".into(),
    };

    std::thread::spawn(move || {
        protocol.announce(&console)
            .expect("Failed announcing network-transfer service");
    });

    
    let app = Router::new()
        .route("/col/metadata", get(get_metadata))
        .route("/col/content/:filename", get(get_content))
        .fallback(fallback_handler);

    log::info!("Running HTTP Server @ {bind_addr}:{SERVER_PORT}");
    // run it with hyper on localhost:3000
    axum::Server::bind(&SocketAddr::new(IpAddr::V4(bind_addr), SERVER_PORT))
        .serve(app.into_make_service())
        .await?;

    Ok(())
}


/*
Handlers
*/
async fn fallback_handler(request: Request<Body>) {
    dbg!(request);
}

/// Get metadata
/// 
/// ```
/// ORIGINAL
/// < HTTP/1.1 200 OK
/// < Content-Type: text/json
/// < Server: Microsoft-HTTPAPI/2.0
/// < Date: Sun, 08 Oct 2023 00:33:34 GMT
/// < Content-Length: 141092
/// 
/// OWN
/// < HTTP/1.1 200 OK
/// < content-type: text/json
/// < server: Microsoft-HTTPAPI/2.0
/// < content-length: 469
/// < date: Sun, 08 Oct 2023 00:35:53 GMT
/// ```
async fn get_metadata(headers: HeaderMap) -> impl IntoResponse {
    dbg!(headers);

    let body = Json(json!(Metadata {
        items: vec![
            MetadataItem {
                typ: "app".into(),
                is_xvc: None,
                has_content_id: false,
                content_id: String::new(),
                product_id: String::new(),
                package_family_name: "11032Reconco.XboxControllerTester_thvmwcgtjwwvy".into(),
                one_store_product_id: "9NBLGGH4PNC7".into(),
                version: "0".into(),
                size: 0,
                allowed_product_id: "".into(),
                allowed_package_family_name: "".into(),
                path: "/col/content/%767601B6E-0294-4007-8682-BEBFBE676320%7D%2311032Reconco.XboxControllerTester_thvmwcgtjwwvy".into(),
                availability: "available".into(),
                generation: "uwpgen9".into(),
                related_media: vec![],
                related_media_family_names: vec![]
            }
        ]
    }));

    (
        [
            ("Content-type", "text/json"),
            ("Server", "Microsoft-HTTPAPI/2.0")
        ],
        body
    )
}

/// Get content
/// 
/// ```
/// ORIGINAL
/// 
/// < HTTP/1.1 206 OK
/// < Content-Type: application/octet-stream
/// < Content-Range: bytes 0-0/105205760
/// < Server: Microsoft-HTTPAPI/2.0
/// < Date: Sun, 08 Oct 2023 00:18:01 GMT
/// < Content-Length: 1
/// 
/// OUR
/// 
/// < HTTP/1.1 206 Partial Content
/// < content-range: bytes 0-0/5242880
/// < content-length: 1
/// < accept-ranges: bytes
/// < content-type: application/octet-stream
/// < server: Microsoft-HTTPAPI/2.0
/// < date: Sun, 08 Oct 2023 00:27:08 GMT
/// ```
async fn get_content(Path(filename): Path<String>, range: Option<TypedHeader<Range>>) -> impl IntoResponse
{
    dbg!(&filename, &range);

    let (_drive_id, filename) = {
        let mut pair = filename.split('#');
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
}