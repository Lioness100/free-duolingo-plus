//! A simple CLI tool to create dummy accounts with referral links to give yourself
//! free Plus (max 24/41 weeks).
//!
//! > ⚠️ A VPN must used to run this tool as Duolingo will not
//! > consider accounts created with the same IP as the original towards the referral
//! > program.
//!
//! > ⚠️ Use at your own risk.
//!
//! ## Usage
//!
//! Follow [these
//! instructions](https://support.duolingo.com/hc/en-us/articles/4404225309581-How-does-the-referral-program-work-)
//! to get your referral code.
//!
//! ```sh
//! free-duolingo-plus --help
//! free-duolingo-plus --code="ASDFGHJKL1234567890QWERTY"
//! free-duolingo-plus --code="ASDFGHJKL1234567890QWERTY" --num=3
//! free-duolingo-plus -c="ASDFGHJKL1234567890QWERTY" --n=3
//! ```

use clap::{value_parser, AppSettings, Parser};
use console::style;
use futures::future;
use indicatif::{ProgressBar, ProgressStyle};

pub mod duo_api;
use crate::duo_api::DuoApi;

/// Struct used to resolve CLI arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, global_setting(AppSettings::DeriveDisplayOrder))]
struct Args {
    #[clap(
        short,
        long,
        help = "The referral code",
        value_parser = DuoApi::is_valid_code
    )]
    code: String,

    #[clap(
        short,
        long,
        help = "The number of accounts to generate (max 24)",
        default_value_t = 24,
        value_parser = value_parser!(u8).range(1..=24)
    )]
    num: u8,
}

/// CLI entrypoint.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let client = DuoApi::new();

    let bar = ProgressBar::new(args.num.into()).with_style(
        ProgressStyle::default_bar().template("[{elapsed_precise}] [{pos}/{len}] {bar:70.cyan}"),
    );

    let futures = (0..args.num).map(|_| {
        let code = args.code.clone();
        let client = client.clone();
        let bar = bar.clone();

        tokio::spawn(async move {
            let data = client
                .create_account(&code)
                .await
                .expect("Failed to create account");

            client
                .create_credentials(&data)
                .await
                .expect("Failed to create credentials");

            bar.inc(1);
        })
    });

    future::try_join_all(futures).await?;
    bar.finish();

    println!(
        "All accounts created! Enjoy your {} weeks of free Duolingo Plus.\n{}",
        style(args.num).green().bold(),
        style("https://www.duolingo.com/").dim()
    );

    Ok(())
}
