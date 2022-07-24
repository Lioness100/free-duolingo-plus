//! A simple CLI tool to create dummy accounts with referral links to give yourself
//! free Plus (max 24 weeks). A VPN must used to run this tool as Duolingo will not
//! consider accounts created with the same IP as the original towards the referral
//! program. This tool is untested and should be used at your own risk.
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

use std::time::Instant;

use clap::{value_parser, AppSettings, Parser};
use indicatif::ProgressBar;

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
    let now = Instant::now();
    let bar = ProgressBar::new(args.num.into());
    let client = DuoApi::new();

    for _ in 1..=args.num {
        // To setup an account, you first need to create it, and then send a
        // patch request to create credentials (which obviously won't be used).
        let data = client.create_account(args.code.to_owned()).await?;
        client.create_credentials(data).await?;

        bar.inc(1);
    }

    bar.finish_with_message(format!("All accounts created in {:.2?}!", now.elapsed()));
    Ok(())
}
