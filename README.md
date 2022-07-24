# Free Duolingo Plus

A simple CLI tool to create dummy accounts with referral links to give yourself
free Plus (max 19 weeks).

> ⚠️ A VPN must used to run this tool as Duolingo will not
> consider accounts created with the same IP as the original towards the referral
> program.

> ⚠️ Use at your own risk.

## Installation

Install [Rust](https://www.rust-lang.org/tools/install) using the recommended
rustup installation method and then run:

```sh
cargo install free-duolingo-plus
```

## Usage

Follow [these
instructions](https://support.duolingo.com/hc/en-us/articles/4404225309581-How-does-the-referral-program-work-)
to get your referral code.

```sh
free-duolingo-plus --help
free-duolingo-plus --code="ASDFGHJKL1234567890QWERTY"
free-duolingo-plus --code="ASDFGHJKL1234567890QWERTY" --num=3
free-duolingo-plus -c="ASDFGHJKL1234567890QWERTY" --n=3
```

!["You earned 19 weeks of free Duoling
Plus!"](https://user-images.githubusercontent.com/65814829/180655454-56f8855f-b279-4509-af50-d7c91ec41530.png)

Please ⭐ this repository if this works for you!

## Acknowledgements

The strategy was taken from https://github.com/JunkMeal/duolingo-plus and ported
to Rust.

## Contributing

This is the first project I've ever created with Rust. If you would like to
improve the code, please open an issue or pull request!

## [Internal Documentation](https://docs.rs/free-duolingo-plus)
