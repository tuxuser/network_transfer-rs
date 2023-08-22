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
mod models;

use std::{time::Duration, net::Ipv4Addr, sync::mpsc::RecvTimeoutError};
use thiserror::Error;

use mdns_sd::{ServiceDaemon, ServiceEvent, ServiceInfo};
use url::Url;

#[derive(Error, Debug)]
pub enum NetworkTransferError {
    #[error("MDNS Error")]
    MdnsError(#[from] mdns_sd::Error),
    #[error("HTTP Error")]
    HttpError(#[from] ureq::Error),
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[error("Timeout Error")]
    TimeoutError(#[from] RecvTimeoutError),
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

    fn build_service_info(console_info: &Console) -> Result<ServiceInfo, NetworkTransferError> {
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

    pub fn discover(&self) -> Result<Vec<Console>, NetworkTransferError> {
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
                    eprintln!(
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

    pub fn announce(&self, console_info: &Console) -> Result<(), NetworkTransferError> {
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
 
    pub fn get_metadata(&self) -> Result<models::Metadata, NetworkTransferError> {
        let url = self.get_url("/col/metadata");

        let resp = self.client
            .get(url.as_ref())
            .call()?
            .into_json::<models::Metadata>()?;

        Ok(resp)
    }

    pub fn download(&self, path: &str) -> Result<ureq::Response, NetworkTransferError> {
        let url = self.get_url(path);
        let resp = self.client
            .get(url.as_ref())
            .set("range", "bytes=0-0")
            .call()?;

        Ok(resp)
    }

    pub fn download_item(&self, item: &models::MetadataItem) -> Result<ureq::Response, NetworkTransferError> {
        self.download(&item.path)
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
}
