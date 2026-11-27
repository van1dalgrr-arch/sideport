use anyhow::{Context, Result};
use std::fs::File;
use std::io::Read;
use zip::ZipArchive;
pub fn read_info(path: &str) -> Result<()> {
    let bundle_id = bundle_id(path)?;
    println!("bundle id: {bundle_id}");
    Ok(())
}

pub fn bundle_id(path: &str) -> Result<String> {
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

    dict.get("CFBundleIdentifier")
        .and_then(|v| v.as_string())
        .map(|s| s.to_string())
        .context("no CFBundleIdentifier")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::write::ZipWriter;

    fn make_test_ipa(path: &std::path::Path, bundle_id: &str) {
        let plist = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>
    <string>{bundle_id}</string>
</dict>
</plist>"#
        );
        let file = File::create(path).unwrap();
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file("Payload/Test.app/Info.plist", options)
            .unwrap();
        zip.write_all(plist.as_bytes()).unwrap();
        zip.finish().unwrap();
    }

    #[test]
    fn test_bundle_id_reads_correct_value() {
        let dir = std::env::temp_dir().join("sideport-test");
        std::fs::create_dir_all(&dir).unwrap();
        let ipa_path = dir.join("test.ipa");
        make_test_ipa(&ipa_path, "com.example.testapp");

        let result = bundle_id(ipa_path.to_str().unwrap());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "com.example.testapp");

        std::fs::remove_file(&ipa_path).ok();
    }

    #[test]
    fn test_bundle_id_missing_file() {
        let result = bundle_id("/nonexistent/path.ipa");
        assert!(result.is_err());
    }
}
