# KDC — Kubernetes Docker Commander

A project-centric DevOps terminal application for managing Docker and Kubernetes workflows from a keyboard-first TUI.

---

## Installation

### macOS — Homebrew (recommended)

If a Homebrew tap is configured, install with:

```bash
brew install KDM-cli/tap/kdc
```

Or install manually from the release tarball:

```bash
# Apple Silicon (M1/M2/M3)
curl -LO https://github.com/KDM-cli/kdc-cli/releases/latest/download/kdc-v0.1.0-aarch64-apple-darwin.tar.gz
tar -xzf kdc-v0.1.0-aarch64-apple-darwin.tar.gz
sudo mv kdc /usr/local/bin/

# Intel
curl -LO https://github.com/KDM-cli/kdc-cli/releases/latest/download/kdc-v0.1.0-x86_64-apple-darwin.tar.gz
tar -xzf kdc-v0.1.0-x86_64-apple-darwin.tar.gz
sudo mv kdc /usr/local/bin/
```

Replace `v0.1.0` with the latest release tag (e.g. `0.1.0`). Find all releases at the [releases page](https://github.com/KDM-cli/kdc-cli/releases).

---

### Linux — Debian / Ubuntu (.deb)

```bash
# amd64
curl -LO https://github.com/KDM-cli/kdc-cli/releases/latest/download/kdc_v0.1.0_amd64.deb
sudo dpkg -i kdc_v0.1.0_amd64.deb

# arm64
curl -LO https://github.com/KDM-cli/kdc-cli/releases/latest/download/kdc_v0.1.0_arm64.deb
sudo dpkg -i kdc_v0.1.0_arm64.deb
```

### Linux — Fedora / RHEL / openSUSE (.rpm)

```bash
# x86_64
sudo rpm -i https://github.com/KDM-cli/kdc-cli/releases/latest/download/kdc-v0.1.0-1.x86_64.rpm

# aarch64
sudo rpm -i https://github.com/KDM-cli/kdc-cli/releases/latest/download/kdc-v0.1.0-1.aarch64.rpm
```

### Linux — tarball (any distro)

```bash
# x86_64
curl -LO https://github.com/KDM-cli/kdc-cli/releases/latest/download/kdc-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
tar -xzf kdc-v0.1.0-x86_64-unknown-linux-gnu.tar.gz
sudo mv kdc /usr/local/bin/

# aarch64
curl -LO https://github.com/KDM-cli/kdc-cli/releases/latest/download/kdc-v0.1.0-aarch64-unknown-linux-gnu.tar.gz
tar -xzf kdc-v0.1.0-aarch64-unknown-linux-gnu.tar.gz
sudo mv kdc /usr/local/bin/
```

---

### Windows

Download the `.zip` from the [releases page](https://github.com/KDM-cli/kdc-cli/releases/latest):

```
kdc-v0.1.0-x86_64-pc-windows-msvc.zip
```

Extract `kdc.exe` and place it somewhere on your `PATH` (e.g. `C:\Users\<you>\bin\`).

---

### Verify the download (optional)

Every release includes a `sha256sums.txt` file. To verify your download:

```bash
# Download the checksum file
curl -LO https://github.com/KDM-cli/kdc-cli/releases/latest/download/sha256sums.txt

# Verify (Linux/macOS)
sha256sum --check --ignore-missing sha256sums.txt
```

---

### Build from source

Requires [Rust](https://rustup.rs/) (stable toolchain).

```bash
git clone https://github.com/KDM-cli/kdc-cli.git
cd kdc-cli
cargo build --release
sudo mv target/release/kdc /usr/local/bin/
```

---

## Quick start

```bash
kdc             # launch the TUI dashboard
kdc scan        # scan the current directory for projects
kdc menus       # print the generated dynamic menu tree
```

---

## Development

```bash
cargo run
cargo run -- scan
cargo run -- menus
cargo test
```