//! A simple CLI tool to create dummy accounts with referral links to give yourself
//! free Plus (max 24/41 weeks).
//!
//! > **⚠️ A VPN must used to run this tool as Duolingo will not
//! > consider accounts created with the same IP as the original towards the referral
//! > program.**
//!
//! ## Usage
//!
//! Follow [these
//! instructions](https://support.duolingo.com/hc/en-us/articles/4404225309581-How-does-the-referral-program-work-)
//! to get your referral link.
//!
//! ```sh
//! free-duolingo-plus --help
//! free-duolingo-plus --code BDHTZTB5CWWKTVW2UCDTY27MBE
//! free-duolingo-plus --code https://invite.duolingo.com/BDHTZTB5CWWKTVW2UCDTY27MBE
//! free-duolingo-plus --code https://invite.duolingo.com/BDHTZTB5CWWKTVW2UCDTY27MBE --num 10
//! ```

use clap::{value_parser, AppSettings, Parser};
use console::style;
use indicatif::{ProgressBar, ProgressIterator, ProgressStyle};

pub mod duo_api;
use crate::duo_api::DuoApi;

/// Struct used to resolve CLI arguments.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None, global_setting(AppSettings::DeriveDisplayOrder))]
struct Args {
    #[clap(
        short,
        long,
        help = "The referral code or link",
        value_parser = DuoApi::parse_code
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
fn main() {
    let args = Args::parse();
    let client = DuoApi::default();

    let bar_style = ProgressStyle::default_bar() //
        .template("[{elapsed_precise}] [{pos}/{len}] {bar:70.cyan}");

    let bar = ProgressBar::new(args.num.into()).with_style(bar_style);

    for _ in ProgressIterator::progress_with(1..=args.num, bar) {
        let data = client.create_account(&args.code);
        client.create_credentials(&data);
    }

    println!(
        "All accounts created! Enjoy your {} weeks of free Duolingo Plus.\n{}",
        style(args.num).green().bold(),
        style("https://www.duolingo.com/").dim()
    );
}
