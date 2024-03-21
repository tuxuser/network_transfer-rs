///
/// Resolved a new service: X31299B15E854._xboxcol._tcp.local. ->  
/// ServiceInfo {
///     ty_domain: "_xboxcol._tcp.local.",
///     sub_domain: None,
///     fullname: "X31299B15E854._xboxcol._tcp.local.",
///     server: "XBOX.local.",
///     addresses: {
///         10.0.0.229
///     },
///     port: 10248,
///     host_ttl: 120,
///     other_ttl: 4500,
///     priority: 0,
///     weight: 0,
///     txt_properties: TxtProperties { properties: [
///         TxtProperty {key: "N", val: Some("XBOX")},
///         TxtProperty {key: "U", val: Some("X31299B15E854")}
///     ]},
///     last_update: 1681568044484,
///     addr_auto: false
/// }
///
///
pub mod models;
pub mod error;

use std::{time::Duration, net::Ipv4Addr};
use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use rand::{thread_rng, Rng};
use url::Url;
use crate::error::Error;

//const CHUNK_SIZE: usize = 0x1000;

pub fn generate_random_console_id() -> String {
    let arr1: [u8; 6] = thread_rng().gen();
    format!("X{}", hex::encode(arr1))
}

#[derive(Debug, PartialEq, Eq, Default)]
pub struct Range {
    first_byte: usize,
    last_byte: usize,
}

impl Range {
    pub fn new(first: usize, last: usize) -> Self {
        Self {
            first_byte: first,
            last_byte: last
        }
    }

    pub fn count(&self) -> usize {
        self.last_byte - self.first_byte + 1
    }
}

#[derive(Debug)]
pub struct NetworkTransferProtocol {}

#[derive(Debug)]
pub struct Console {
    pub address: Ipv4Addr,
    pub port: u16,
    pub id: String,
    pub name: String,
}

impl From<ServiceInfo> for Console {
    fn from(value: ServiceInfo) -> Self {
        Self {
            address: *value.get_addresses().iter().next().unwrap(),
            port: value.get_port(),
            id: value.get_property_val_str("U").unwrap().to_string(),
            name: value.get_property_val_str("N").unwrap().to_string(),
        }
    }
}

impl NetworkTransferProtocol {
    pub const SERVICE_TYPE: &'static str = "_xboxcol._tcp.local.";
    pub const SERVICE_PORT: u16 = 10248;

    fn build_service_info(console_info: &Console) -> Result<ServiceInfo, Error> {
        let hostname = console_info.name.to_string() + ".local.";
        let properties = [("N", &console_info.name), ("U", &console_info.id)];

        Ok(ServiceInfo::new(
            Self::SERVICE_TYPE,
            &console_info.name,
            &hostname,
            console_info.address,
            console_info.port,
            &properties[..],
        )?)
    }

    pub fn discover(&self) -> Result<Vec<Console>, Error> {
        let mdns = ServiceDaemon::new()?;

        // Browse for a service type.
        let receiver = mdns.browse(Self::SERVICE_TYPE)?;

        let (tx, rx) = std::sync::mpsc::channel();

        // Receive the browse events in sync or async. Here is
        // an example of using a thread. Users can call `receiver.recv_async().await`
        // if running in async environment.
        std::thread::spawn(move || {
            while let Ok(event) = receiver.recv() {
                if let ServiceEvent::ServiceResolved(info) = event {
                    log::info!(
                        "Resolved a new service: {} -> {:?}",
                        info.get_fullname(),
                        info
                    );
                    tx.send(info).unwrap();
                }
            }
        });

        let result = rx.recv_timeout(Duration::from_secs(60))?;

        Ok(vec![Console::from(result)])
    }

    pub fn announce(&self, console_info: &Console) -> Result<(), Error> {
        // Create a daemon
        let mdns = ServiceDaemon::new()?;

        let service_info = Self::build_service_info(console_info)?;

        // Register with the daemon, which publishes the service.
        mdns.register(service_info)?;

        std::thread::sleep(Duration::from_secs(60 * 5));
        Ok(())
    }
}


pub struct Client {
    address: String,
    port: u16,
    client: ureq::Agent,
}

impl From<&Console> for Client {
    fn from(value: &Console) -> Self {
        Self::new(&value.address.to_string(), value.port)
    }
}

impl Client {
    pub fn new(address: &str, port: u16) -> Self {
        let agent = ureq::builder()
            .user_agent("CopyOnLanSvc")
            .build();

        Self {
            address: address.to_string(),
            port,
            client: agent,
        }
    }

    fn get_url(&self, path: &str) -> Url {
        let host = format!("http://{}:{}", self.address, self.port);
        let mut url = Url::parse(&host).unwrap();
        url.set_path(path);

        url
    }
 
    pub fn get_metadata(&self) -> Result<models::Metadata, Error> {
        let url = self.get_url("/col/metadata");

        let resp = self.client
            .get(url.as_ref())
            .set("Accept", "application/json")
            .set("user-agent", "CopyOnLanSvc")
            .set("x-contract-version", "1")
            .call()
            .map_err(Box::new)?
            .into_json::<models::Metadata>()?;

        Ok(resp)
    }

    pub fn iterate_range(size: usize, step_size: usize) -> impl Iterator<Item = Range> {
        (0..size).step_by(step_size).map(move |offset| {
            let size = std::cmp::min(step_size, size - offset);
            Range {
                first_byte: offset,
                last_byte: offset + size - 1
            }
        })
    }

    pub fn download_chunk(&self, path: &str, range: &Range) -> Result<ureq::Response, Error> {
        let url = self.get_url(path);

        let resp = self.client
            .get(url.as_ref())
            .set("range", &format!("bytes={}-{}", range.first_byte, range.last_byte))
            .call()
            .map_err(Box::new)?;

        Ok(resp)
    }

    pub fn get_item_filesize(&self, item: &models::MetadataItem) -> Result<usize, Error> {
        let resp = self.download_chunk(&item.path, &Range::default())?;
        log::trace!("{resp:?}");
        let headers: Vec<String> = resp
        .headers_names()
        .into_iter()
        .map(|k| {
            let hdr_name = k.clone();
            format!("{k}: {}", resp.header(hdr_name.as_ref()).unwrap_or("<NOT_SET>"))
        }).collect();

        dbg!(headers);
        assert_eq!(resp.status(), 206, "Unexpected HTTP status, expected 206");

        let content_length = match resp.header("content-range") {
            Some(content_range) => {
                let content_length = content_range.split('/')
                    .last()
                    .ok_or(Error::GeneralError("Failed to get full content length".to_owned()))?;

                content_length.parse::<usize>()
                    .map_err(|_|Error::GeneralError("Failed to parse content length".to_owned()))?
            },
            None => Err(Error::GeneralError("No content-range header returned".to_owned()))?,
        };

        Ok(content_length)
    }

    pub fn download_chunks(&self, item: &models::MetadataItem, content_length: usize, writer: &mut impl std::io::Write, chunk_size: usize) -> Result<usize, Error>  {
        dbg!(content_length);

        let mut buf = vec![0u8; chunk_size];
        let written: usize = Self::iterate_range(content_length, chunk_size).into_iter().map(move |range|{
            let resp = self.download_chunk(&item.path, &range).unwrap();
            resp.into_reader().read_exact(&mut buf).unwrap();
            writer.write(&buf).unwrap();

            range.count()
        }).sum();

        assert_eq!(written, content_length);
        Ok(written)
    }
}

struct Server;

impl Server {

}

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use super::*;

    #[test]
    fn test_build_service_info() {
        let console = Console {
            address: Ipv4Addr::new(1, 2, 3, 4),
            port: 4321,
            id: "X92348235235".to_string(),
            name: "TESTXBOX".to_string(),
        };

        let info = NetworkTransferProtocol::build_service_info(&console)
            .expect("Failed to create service info");

        assert_eq!(info.get_type(), "_xboxcol._tcp.local.");
        assert!(info
            .get_addresses()
            .get(&Ipv4Addr::new(1, 2, 3, 4))
            .is_some());
        assert!(info
            .get_addresses()
            .get(&Ipv4Addr::new(2, 2, 2, 2))
            .is_none());
        assert_eq!(info.get_fullname(), "TESTXBOX._xboxcol._tcp.local.");
        assert_eq!(info.get_hostname(), "TESTXBOX.local.");
        assert_eq!(info.get_port(), 4321);
        assert_eq!(info.get_properties().len(), 2);
        assert_eq!(info.get_property_val_str("N"), Some("TESTXBOX"));
        assert_eq!(info.get_property_val_str("U"), Some("X92348235235"));
    }

    #[test]
    fn test_range_iterator() {
        let mut it1 = Client::iterate_range(4200, 1024);
        assert_eq!(Range::new(0,1023), it1.next().unwrap());
        assert_eq!(Range::new(1024,2047), it1.next().unwrap());
        assert_eq!(Range::new(2048,3071), it1.next().unwrap());
        assert_eq!(Range::new(3072,4095), it1.next().unwrap());
        assert_eq!(Range::new(4096,4199), it1.next().unwrap());
    }
}
