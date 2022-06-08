<p align="center">
  <img alt="Memorage logo" src="./static/logo-light.svg#gh-light-mode-only" />
  <img alt="Memorage logo" src="./static/logo-dark.svg#gh-dark-mode-only" />
</p>

**Memorage** is a peer to peer backup service. Set up with a friend to store
automatic and encrypted backups on each other's computer.

## Installation

**Memorage** must be built from source using the nightly Rust toolchain:
```bash
git clone https://github.com/tsoutsman/memorage
cd memorage/crates/client-cli
cargo install --path .
````

The nightly Rust toolchain can be installed with the following command:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

## Features

- Per file backup - only modified files have to be re-encrypted and resent
- Authentication using ED25519 keys
- XChaCha20Poly1305 encryption for backups
- CLI

### Planned

- Historical (incremental) backups
- Multiple peers
- GUI
- Proof of capacity mode

## FAQ

### How do peers establish a connection?
Clients rely on a synchronisation server (which can be self-hosted) to decide
when to connect. After they connect, the synchronisation server is no longer 
needed, as the peers can send data directly to each other. <!-- TODO link page
about self hosting -->

<!-- ### How to Use -->
