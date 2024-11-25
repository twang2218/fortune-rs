 # fortune-rs 🎲

[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)
[![Crates.io](https://img.shields.io/crates/v/fortune-rs.svg)](https://crates.io/crates/fortune-rs)
[![Downloads](https://img.shields.io/crates/d/fortune-rs.svg)](https://crates.io/crates/fortune-rs)
[![Documentation](https://docs.rs/fortune-rs/badge.svg)](https://docs.rs/fortune-rs)
[![build](https://github.com/twang2218/fortune-rs/actions/workflows/build.yml/badge.svg)](https://github.com/twang2218/fortune-rs/actions/workflows/build.yml)
[![codecov](https://codecov.io/gh/twang2218/fortune-rs/branch/main/graph/badge.svg)](https://codecov.io/gh/twang2218/fortune-rs)
[![dependency status](https://deps.rs/repo/github/twang2218/fortune-rs/status.svg)](https://deps.rs/repo/github/twang2218/fortune-rs)
[![MSRV](https://img.shields.io/badge/MSRV-1.70.0-blue)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](http://makeapullrequest.com)

> A modern, feature-rich implementation of the classic BSD `fortune` program in Rust. This implementation aims to be a drop-in replacement for traditional `fortune` programs on Unix-like systems while maintaining compatibility with various fortune database formats.

## 📑 Table of Contents

- [fortune-rs 🎲](#fortune-rs-)
  - [📑 Table of Contents](#-table-of-contents)
  - [✨ Features](#-features)
  - [📥 Installation](#-installation)
    - [From Source](#from-source)
  - [🚀 Usage](#-usage)
    - [Basic Usage](#basic-usage)
    - [Common Options](#common-options)
    - [Advanced Usage](#advanced-usage)
  - [🛠 Development](#-development)
    - [Project Structure](#project-structure)
    - [Building](#building)
    - [Testing](#testing)
      - [Test Coverage](#test-coverage)
  - [🔧 Implementation Details](#-implementation-details)
  - [🗺 Roadmap](#-roadmap)
    - [Current Status](#current-status)
    - [Future Plans](#future-plans)
  - [👥 Contributing](#-contributing)
  - [📄 License](#-license)
  - [📚 References](#-references)
    - [Rust Implementations](#rust-implementations)

## ✨ Features

- 🔄 Full compatibility with traditional fortune program options
- 📚 Support for multiple fortune database formats
- 🔍 Pattern matching with regular expressions
- 📊 Weighted fortune selection
- 🌳 Recursive directory searching
- 📏 Precise control over fortune length
- 🎨 Support for both regular and offensive fortunes
- 🐛 Debug output for troubleshooting

## 📥 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/twang2218/fortune-rs.git

# Change into the directory
cd fortune-rs

# Build with optimizations
cargo build --release
```

> The compiled binary will be available at `target/release/fortune`

## 🚀 Usage

### Basic Usage

```bash
# Display a random fortune
fortune

# Display a random fortune from specific files or directories
fortune /path/to/fortune/file

# Display a random offensive fortune
fortune -o
```

### Common Options

| Option | Description |
|--------|-------------|
| `-a` | Choose from all lists of maxims |
| `-c` | Show the cookie file source |
| `-f` | Print out the list of files |
| `-o` | Choose only offensive fortunes |
| `-s` | Display short fortunes only |
| `-l` | Display long fortunes only |
| `-n length` | Set length cutoff |
| `-m pattern` | Display matching fortunes |
| `-i` | Ignore case in pattern matching |
| `-w` | Wait based on message length |
| `-e` | Equal size file handling |
| `-D` | Enable debugging output |

### Advanced Usage

```bash
# Pattern matching (case-insensitive)
fortune -i -m "pattern"

# Short fortunes only
fortune -s

# Show fortune sources
fortune -c

# List available fortune files
fortune -f

# Weighted selection
fortune 30% /path/to/fortunes1 70% /path/to/fortunes2
```

## 🛠 Development

### Project Structure

```
fortune-rs/
├── src/
│   ├── fortune.rs    # Main implementation
│   ├── strfile.rs    # Database generator
│   └── metadata.rs   # Metadata handling
├── tests/
│   ├── integration.rs # Integration tests
│   ├── data/         # Test files
│   └── data2/        # Additional tests
└── Cargo.toml        # Project manifest
```

### Building

```bash
# Debug build
cargo build

# Release build with optimizations
cargo build --release
```

### Testing

```bash
# Run all tests
cargo test
```

#### Test Coverage

- ✅ Pattern matching (`-m`)
- ✅ Case-insensitive search (`-i`)
- ✅ Length-based filtering (`-l`, `-s`, `-n`)
- ✅ File listing (`-f`)
- ✅ Weighted selection
- ✅ Offensive fortunes (`-o`)
- ✅ Equal-size handling (`-e`)

## 🔧 Implementation Details

- ✅ Compatible with traditional fortune database formats
- ✅ Supports regular and offensive fortunes
- ✅ Implements weighted selection
- ✅ Pattern matching with regex
- ✅ Recursive directory traversal
- ✅ Multiple file formats and encodings
- ✅ Strfile index compatibility

## 🗺 Roadmap

### Current Status

- ✅ Core Functionality
  - ✅ Random fortune display
  - ✅ Strfile index support
  - ✅ Directory searching

- ✅ Traditional Options
  - ✅ Standard flags
  - ✅ Pattern matching
  - ✅ Length control
  - ✅ Debug output

### Future Plans

- 🔄 Modern Enhancements
  - ⏳ TOML configuration
  - ⏳ Embedded fortune database

- 📈 Project Growth
  - ⏳ Extended docs
  - ✅ Comprehensive testing
  - ⏳ CI/CD pipeline

## 👥 Contributing

We welcome contributions! Here's how you can help:

1. 🍴 [Fork](https://github.com/twang2218/fortune-rs/fork) the repository
2. 🌿 [Create a branch](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-and-deleting-branches-within-your-repository) for your feature
3. ✅ [Add tests](https://doc.rust-lang.org/book/ch11-01-writing-tests.html) for new features
4. 🧪 [Ensure all tests pass](https://doc.rust-lang.org/book/ch11-00-testing.html)
5. 📬 [Submit a pull request](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📚 References

### Rust Implementations

- [cmatsuoka/fortune-rs](https://github.com/cmatsuoka/fortune-rs), by Claudio Matsuoka, Brazil, at 2017;
- [c-OO-b/rust-fortune](https://github.com/c-OO-b/rust-fortune), by c-OO-b, Norway, at 2019;
- [wapm-packages/fortune](https://wapm.io/package/fortune), forked from [c-OO-b/rust-fortune](https://github.com/c-OO-b/rust-fortune), at 2019;
- [runebaas/fortune-mod.rs](https://github.com/runebaas/fortune-mod.rs), by [Daan Boerlage](https://boerlage.me), Switzerland, at 2019;
- [kvrohit/fortune](https://github.com/kvrohit/fortune), by [Rohit K Viswanath](https://kvrohit.dev/), at 2020;
- [davidkna/lolcow-fortune-rs](https://github.com/davidkna/lolcow-fortune-rs.git), by David Knaack, Berlin, Germany, at 2021;
- [blackbird1128/fortune_cookie](https://github.com/blackbird1128/fortune_cookie.rs.git), by Alexj, at 2023;
- [zuisong/rs-fortune](https://github.com/zuisong/rs-fortune), by ZuiSong, Changsha, China, at 2023;
- [cafkafk/fortune-kind](https://github.com/cafkafk/fortune-kind), by [Christina Sørensen](https://www.linkedin.com/in/cafkafk/), Denmark, at 2023;
- [rilysh/fortune-day](https://github.com/rilysh/fortune-day.git), at 2024;
- [FaceFTW/shell-toy](https://github.com/FaceFTW/shell-toy.git), by [Alex Westerman](http://faceftw.dev/), at 2024;
