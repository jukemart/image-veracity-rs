# Image - Veracity Project

## Setup

Requires Rust (MSRV 1.62) and Protobuf i.e. `protoc` installed on the local system.

Obtain Rust via [rustup](https://rustup.rs/) (requires cUrl, more information found [here](https://www.rust-lang.org/learn/get-started)):

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

rustup update
```

Protobuf support can be installed via package manager for your operating system:

### Arch Linux
```shell
sudo pacman -Syu
sudo pacman -S protobuf
```

### Debian / Ubuntu
```shell
sudo apt install protobuf-compiler
```

### Fedora / RHEL
```shell
sudo dnf -y install protobuf
```

Additional information can be found on the [Protobuf documentation page](https://protobuf.dev/).

Ensure `protoc` is installed:

```shell
protoc --version
```