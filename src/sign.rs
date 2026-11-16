use std::fs::File;
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use isideload::dev::developer_session::{DeveloperSession, DevicesApi};
use isideload::sideload::SideloaderBuilder;
use isideload::sideload::sideloader::Sideloader;
use isideload::util::callbacks::{ExtensionsBehaviorCallbackBox, MaxCertsCallbackBox};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

use isideload::anisette::remote_v3::RemoteV3AnisetteProvider;
use isideload::auth::apple_account::AppleAccount;
use isideload::auth::apple_account::{TwoFactorCallbackParams, TwoFactorCallbackResponse};
use rootcause::Report;
use rootcause::compat::IntoRootcause;

/// Prints `prompt` and reads one line from the terminal.
///
/// ```ignore
/// let email = ask("Enter your email: ")?;
/// ```
fn ask(prompt: &str) -> Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;

    let mut line = String::new();
    io::stdin().read_line(&mut line)?;

    Ok(line.trim().to_string())
}

async fn sign_in() -> Result<(AppleAccount, String)> {
    let apple_id = ask("Apple ID: ")?;
    let password = rpassword::prompt_password("Password: ")?;

    if apple_id.is_empty() {
        bail!("Apple ID is empty");
    }
    if password.is_empty() {
        bail!("password is empty");
    }

    let account = AppleAccount::builder(&apple_id)
        .anisette_provider(RemoteV3AnisetteProvider::default()?)
        .login(&password, two_factor)
        .await?;

    Ok((account, apple_id))
}

pub async fn login() -> Result<()> {
    let (account, _) = sign_in().await?;

    let (first, last) = account.get_name()?;
    println!("logged in as {first} {last}");

    Ok(())
}

pub async fn sign_ipa(ipa: &str, device_name: &str, udid: &str) -> Result<PathBuf> {
    let (mut account, apple_id) = sign_in().await?;
    let session = DeveloperSession::from_account(&mut account).await?;

    let mut sideloader: Sideloader<MaxCertsCallbackBox, ExtensionsBehaviorCallbackBox> =
        SideloaderBuilder::new(session, apple_id)
            .machine_name("sideport".to_string())
            .build();

    let team = sideloader.get_team().await?;
    println!("team: {}", team.name.as_deref().unwrap_or("?"));

    sideloader
        .get_dev_session()
        .ensure_device_registered(&team, device_name, udid, None)
        .await?;

    println!("signing...");
    let (app_dir, _) = sideloader
        .sign_app(
            PathBuf::from(ipa),
            Some(team),
            false,
            None::<fn(f32) -> std::future::Ready<()>>,
            None,
            None,
        )
        .await?;

    println!("packing...");
    let signed = std::env::temp_dir().join("sideport-signed.ipa");
    let target = signed.clone();
    tokio::task::spawn_blocking(move || pack(&app_dir, &target)).await??;

    Ok(signed)
}

fn pack(app_dir: &Path, ipa: &Path) -> Result<()> {
    let name = app_dir
        .file_name()
        .and_then(|n| n.to_str())
        .context("bad app bundle path")?;

    let file = File::create(ipa).with_context(|| format!("cannot create {}", ipa.display()))?;
    let mut zip = ZipWriter::new(file);

    pack_dir(&mut zip, app_dir, &format!("Payload/{name}"))?;
    zip.finish()?;

    Ok(())
}

fn pack_dir(zip: &mut ZipWriter<File>, dir: &Path, prefix: &str) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = format!("{prefix}/{}", entry.file_name().to_string_lossy());
        let meta = std::fs::symlink_metadata(&path)?;

        let options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(meta.permissions().mode())
            .large_file(meta.len() > u32::MAX as u64);

        if meta.file_type().is_symlink() {
            let target = std::fs::read_link(&path)?;
            zip.add_symlink(name, target.to_string_lossy(), options)?;
        } else if meta.is_dir() {
            pack_dir(zip, &path, &name)?;
        } else {
            zip.start_file(name, options)?;
            let mut source = File::open(&path)?;
            std::io::copy(&mut source, zip)?;
        }
    }

    Ok(())
}

async fn two_factor(_params: TwoFactorCallbackParams) -> Result<TwoFactorCallbackResponse, Report> {
    let code = ask("2FA_code: ").into_rootcause()?;
    Ok(TwoFactorCallbackResponse::SubmitCode(code))
}
