use std::fs::File;
use std::io::Write;
use std::process::Command;
use zip::write::SimpleFileOptions;
use zip::write::ZipWriter;

fn create_test_ipa(path: &std::path::Path, bundle_id: &str) {
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
fn test_info_prints_bundle_id() {
    let dir = std::env::temp_dir().join("sideport-integration-test");
    std::fs::create_dir_all(&dir).unwrap();
    let ipa_path = dir.join("test.ipa");
    create_test_ipa(&ipa_path, "com.example.testapp");

    let output = Command::new(env!("CARGO_BIN_EXE_sideport"))
        .arg("info")
        .arg(ipa_path.to_str().unwrap())
        .output()
        .expect("failed to run sideport");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bundle id: com.example.testapp"));

    std::fs::remove_file(&ipa_path).ok();
}

#[test]
fn test_info_missing_file() {
    let output = Command::new(env!("CARGO_BIN_EXE_sideport"))
        .arg("info")
        .arg("/nonexistent/path.ipa")
        .output()
        .expect("failed to run sideport");

    assert!(!output.status.success());
}
