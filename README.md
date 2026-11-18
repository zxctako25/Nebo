<div align="center">

# Nebo

**A fast Minecraft proxy core written in Rust**

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)
![Minecraft](https://img.shields.io/badge/Minecraft-1.16.5%20→%201.21.11-62B47A?style=for-the-badge)
![Status](https://img.shields.io/badge/status-early%20testing-orange?style=for-the-badge)

</div>

---

> [!WARNING]
> ## Very early and unstable
> **Nebo is in early development.** This is **not a final version**, only a **test build**.
> Expect bugs, crashes, unexpected behavior, and incompatibilities.
> **Do not use it on production servers.** If you find a bug, please open an [issue](../../issues).

---

## About

**Nebo** is a Minecraft proxy core written in **Rust**.
It is built on top of the **1.16.5** protocol and supports clients and servers
from **1.16.5 to 1.21.11**.

The goal: a lightweight, fast proxy that bridges different game versions,
so players don't have to hunt for the exact client version.

## Features

- Written in Rust: fast and memory-efficient
- Supports versions **1.16.5 – 1.21.11**
- Proxy core based on the 1.16.5 protocol
- *(more coming soon, see [Roadmap](#roadmap))*

## Installation

### Requirements

- [Rust](https://www.rust-lang.org/tools/install) (stable)
- Git

### Build from source

```bash
git clone https://github.com/zxctako25/nebo.git
cd nebo
cargo build --release
```

The binary will be in `target/release/`.

## Usage

```bash
cargo run --release
```

> Launch options and configuration docs are coming later.

## Configuration

*Work in progress.*

## Roadmap

- [x] Basic proxy core on 1.16.5
- [x] Support for versions 1.16.5 – 1.21.11
- [ ] Bug fixes and stabilization
- [ ] Documentation and example configs
- [ ] First stable release

## Known issues

Since the project is in testing, you may encounter:

- crashes and disconnects on various versions;
- compatibility problems on specific versions;
- incomplete support for some packets.

Known bugs are tracked in [Issues](../../issues).

## Contributing

Pull requests and bug reports are welcome!

1. Fork the repository
2. Create a branch: `git checkout -b feature/my-feature`
3. Commit your changes: `git commit -m "Add my feature"`
4. Push the branch: `git push origin feature/my-feature`
5. Open a Pull Request

---

<div align="center">

Made with ❤️

</div>
