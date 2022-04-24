# amaze

## Quick Start

```sh
~$ cd amaze
~/amaze$ cargo run -- double 50
```

## Project Structure

This repository is setup as a [cargo workspace][] containing two rust projects:
a library, and a CLI binary that brings it in as a dependency. The intention is
to separate the core functionality from any presentation layer early on.

```
├── Cargo.lock
├── Cargo.toml
├── README.md
├── amaze
│   ├── Cargo.toml
│   └── src
│       └── lib.rs
└── amaze-cli
    ├── Cargo.toml
    └── src
        └── main.rs
```
## About

This repository was generated using [cargo-generate][], with [jcpst/rust-utility-template][] as the template.

```sh
cargo install cargo-generate
cargo generate --git jcpst/rust-utility-template --name amaze
```

<!-- links -->
[cargo workspace]: https://doc.rust-lang.org/cargo/reference/workspaces.html
[cargo-generate]: https://github.com/cargo-generate/cargo-generate
[jcpst/rust-utility-template]: https://github.com/jcpst/rust-utility-template