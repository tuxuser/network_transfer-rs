use std::net::TcpStream;

use anyhow::{Result};
use env_logger::Env;
use tokio::net::TcpListener;

use std::io::{Read,Write};

use hexdump::hexdump;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let listener = TcpListener::bind(("0.0.0.0", 10248)).await?;

    let (instream, addr)  = listener.accept().await?;
    dbg!(addr);

    let mut inbuf = [0u8; 4096 * 10];
    let mut outbuf = vec![0u8; 4096 * 10];

    let mut outstream = TcpStream::connect((addr.ip(), 10248))?;

    loop {
        log::debug!("Trying to read from server socket");
        instream.readable().await?;

        let insize = match instream.try_read(&mut inbuf)
        {
            Ok(n) => {
                n
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => {
                return Err(e.into());
            }
        };
        log::debug!("read request from console, bytes: {}", insize);

        hexdump(&inbuf[..insize]);

        let _ = outstream.write(&inbuf[..insize])?;
        log::debug!("Wrote request to console");

        // let outsize = outstream.read(&mut outbuf)?;
        let outsize = outstream.read_to_end(&mut outbuf)?;
        log::debug!("Read response from console, bytes: {}", outsize);
        instream.try_write(&outbuf[..outsize])?;
        log::debug!("Wrote response to server socket -> console");
    }
}