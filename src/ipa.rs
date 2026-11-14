use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use zip::ZipArchive;
pub fn read_info(path: &str) -> Result<()> {
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut plist_path = None;

    for name in archive.file_names() {
        let name = name?;
        if name.ends_with(".app/Info.plist") && name.matches('/').count() == 2 {
            plist_path = Some(name.to_string());
            break;
        }
    }
    let plist_path = plist_path.context("Info.plist not found")?;

    let mut entry = archive.by_name(&plist_path)?;
    let mut buf = Vec::new();
    entry.read_to_end(&mut buf)?;

    let value: plist::Value = plist::from_bytes(&buf)?;
    let dict = value
        .as_dictionary()
        .context("Info.plist is not a dictionary")?;

    let bundle_id = dict
        .get("CFBundleIdentifier")
        .and_then(|v| v.as_string())
        .context("no CFBundleIdentifier")?;

    println!("bundle id: {bundle_id}");

    Ok(())
}
