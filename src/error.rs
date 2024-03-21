use thiserror::Error;
use std::sync::mpsc::RecvTimeoutError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("MDNS Error")]
    MdnsError(#[from] mdns_sd::Error),
    #[error("HTTP Error")]
    HttpError(#[from] Box<ureq::Error>),
    #[error("IO Error")]
    IoError(#[from] std::io::Error),
    #[error("Timeout Error")]
    TimeoutError(#[from] RecvTimeoutError),
    #[error("GeneralError")]
    GeneralError(String),
}