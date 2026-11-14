use std::env;

use anyhow::{Context, Result};

mod ipa;

fn main() -> Result<()> {
    let path = env::args()
        .nth(1)
        .context("usege: upload .ipa file to resign")?;

    ipa::read_info(&path)?;

    Ok(())
}
