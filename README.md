### This branch creates each profile in parallel, which is much faster but turns out to be quite janky and a lot of te time not all free weeks are registered.

<br />
<p align="center">
  <img
    alt='"You earned 19 weeks of free Duolingo
Plus!"'
    src="https://user-images.githubusercontent.com/65814829/180655454-56f8855f-b279-4509-af50-d7c91ec41530.png"
    width="153.5"
    height="358.5"
  />
  <img
    alt='"You earned 41 weeks of free Duolingo
Plus!"'
    src="https://user-images.githubusercontent.com/65814829/180673748-af68696e-f418-4728-a733-ae3be23b5e94.png"
    width="153.5"
    height="358.5"
  />
  <img
    alt='"You earned 36 weeks of free Duolingo
Plus!"'
    src="https://user-images.githubusercontent.com/65814829/180903063-27544207-f39a-4e57-a9e7-312fcad088ab.png"
    width="153.5"
    height="358.5"
  />
    <img
    alt='"You earned 24 weeks of free Duolingo
Plus!"'
    src="https://user-images.githubusercontent.com/65814829/180903203-92fd105b-c7b9-45a6-b3bd-7f6c90e695a5.png"
    width="153.5"
    height="358.5"
  />
</p>
<br />

# Free Duolingo Plus

A simple CLI tool to create dummy accounts with referral links to give yourself
free Plus (max 24/41 weeks depending on whether you're part of the [tiered
reward system](https://user-images.githubusercontent.com/65814829/180666541-8ceac559-37d8-4e5b-86f4-05b8b265b3b6.png)).

## Installation

> _🎉 If you wouldn't like to go through the installation process, you can
> [create an
> issue](https://github.com/Lioness100/free-duolingo-plus/issues/new?assignees=Lioness100&labels=&template=enter-your-referral-code-link.md&title=Remote+CLI+Usage+Request)
> or reach out to me on Discord (@Lioness100#4566) and I will run the tool on
> your behalf._

Install [Rust](https://www.rust-lang.org/tools/install) using the recommended
rustup installation method and then run:

```sh
cargo install free-duolingo-plus
```

## Usage

Follow [these
instructions](https://support.duolingo.com/hc/en-us/articles/4404225309581-How-does-the-referral-program-work-)
to get your referral code.

⚠️ **A VPN must used to run this as Duolingo will not
consider accounts created with the same IP as the original towards the referral
program.** ⚠️

```sh
free-duolingo-plus --help
free-duolingo-plus --code="ASDFGHJKL1234567890QWERTY"
free-duolingo-plus --code="ASDFGHJKL1234567890QWERTY" --num=3
free-duolingo-plus -c="ASDFGHJKL1234567890QWERTY" --n=3
```

Please ⭐ this repository if it works for you!

## Acknowledgements

The strategy was taken from https://github.com/JunkMeal/duolingo-plus and ported
to Rust.

## Contributing

This is the first project I've ever created with Rust. If you would like to
improve the code, please open an issue or pull request!

## [Internal Documentation](https://docs.rs/free-duolingo-plus)
