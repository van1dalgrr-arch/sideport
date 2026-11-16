use std::env;

use anyhow::{Context, Result};

mod afc;
mod device;
mod ipa;
mod lockdown;
mod service;
mod sign;
mod tls;
mod usbmux;
mod watch;

#[tokio::main()]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let cmd = args.next().context("usage: sideport <info|devices|apps|install|login>")?;

    match cmd.as_str() {
        "info" => {
            let path = args.next().context("usage: sideport info <file.ipa>")?;
            ipa::read_info(&path)?;
        }
        "login" => sign::login().await?,
        "install" => {
            let path = args.next().context("usage: sideport install <file.ipa>")?;
            device::install(&path).await?;
        }
        "watch" => watch::run().await?,
        "apps" => device::apps().await?,
        "devices" => device::list().await?,
        _ => anyhow::bail!("unknown command: {cmd}"),
    }
    Ok(())

}
