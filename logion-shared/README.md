# logion-shared

Defines [traits](https://doc.rust-lang.org/rust-by-example/trait.html) used by more than one pallet.

## Usage

In your project dependencies (`Cargo.toml`):

```toml
[package]

[dependencies]

logion-shared = { default-features = false, version = '0.1.0' }
```

## Build

    carggo build --release

## Publish on crates.io

### Prerequisite

The authentication and publication on [crates.io](https://crates.io/) is done via a github account.

An [API key](https://doc.rust-lang.org/cargo/reference/publishing.html) must be configured.

The publisher of the first version becomes the owner of the crates. In order to be added as owner, use the command:
[cargo owner](https://doc.rust-lang.org/cargo/reference/publishing.html#cargo-owner).

You can also add a team as owner, for instance:

    cargo owner --add github:logion-network:logion-devops-team

### Publication

Increment the version in [Cargo.toml](Cargo.toml):

```toml
[package]

version = '0.2.0'
```

Publish with the following commands (`dry-run` option will perform some checks without actually publishing):

    cargo publish --dry-run

    cargo publish