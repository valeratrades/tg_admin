# tg admin
![Minimum Supported Rust Version](https://img.shields.io/badge/nightly-1.81+-ab6000.svg)
[<img alt="crates.io" src="https://img.shields.io/crates/v/tg_admin.svg?color=fc8d62&logo=rust" height="20" style=flat-square>](https://crates.io/crates/tg_admin)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs&style=flat-square" height="20">](https://docs.rs/tg_admin)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/valeratrades/tg_admin/ci.yml?branch=master&style=for-the-badge&style=flat-square" height="20">](https://github.com/valeratrades/tg_admin/actions?query=branch%3Amaster) <!--NB: Won't find it if repo is private-->
![Lines Of Code](https://img.shields.io/badge/LoC-974-lightblue)

Util to control configuration settings via a telegram bot.

<!-- markdownlint-disable -->
<details>
  <summary>
    <h2>Installation<h2>
  </summary>

Ensure Rust is installed, then cargo-install the binary:
```sh
which rustup || curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install --path . --root /usr/bin/
```
</details>
<!-- markdownlint-restore -->

## Usage
All commands are accessible via a `-h` help message request.
Main use-case is managing a config file that is in use by the target application.
Generalises to all popular config formats.

### Example
```sh
tg_admin watch -t "${THE_BOT_TOKEN}" ./config/config.json
```
// in paths `~` for home directory is supported, but only for linux.

<br>

<sup>
This repository follows <a href="https://github.com/valeratrades/.github/tree/master/best_practices">my best practices</a>.
</sup>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
