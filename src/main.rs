use std::env;

use anyhow::{Context, Result};

mod device;
mod ipa;
mod lockdown;

#[tokio::main()]
async fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let cmd = args.next().context("usage: sideport <info|devices>")?;

    match cmd.as_str() {
        "info" => {
            let path = args.next().context("usage: sideport info <file.ipa>")?;
            ipa::read_info(&path)?;
        }
        "devices" => device::list().await?,
        _ => anyhow::bail!("unknown command: {cmd}"),
    }
    Ok(())
}
