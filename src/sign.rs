use anyhow::{Result, bail};
use std::io::{self, Write};

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

pub async fn login() -> Result<()> {
    let apple_id = ask("AppleID: ")?;
    let password = rpassword::prompt_password("Password: ")?;

    if password.is_empty() {
        bail!("password is empty");
    }

    let account = AppleAccount::builder(&apple_id)
        .anisette_provider(RemoteV3AnisetteProvider::default()?)
        .login(&password, two_factor)
        .await?;

    let (first, last) = account.get_name()?;
    println!("logged as {first} {last}");
    Ok(())
}

async fn two_factor(_params: TwoFactorCallbackParams) -> Result<TwoFactorCallbackResponse, Report> {
    let code = ask("2FA_code: ").into_rootcause()?;
    Ok(TwoFactorCallbackResponse::SubmitCode(code))
}
