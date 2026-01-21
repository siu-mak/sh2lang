# sh2lang v0.1.0

First public release of the sh2 structured shell language.

## What's included

- **sh2c**: Compiler that converts `.sh2` files to bash or POSIX shell scripts
- **sh2do**: Snippet runner for quick one-liners
- **VS Code extension**: Syntax highlighting for `.sh2` files

## Install from source

```bash
git clone https://github.com/siu-mak/sh2lang.git
cd sh2lang
cargo build --workspace
```

### Install to PATH (recommended)

```bash
cargo install --path sh2c --locked
cargo install --path sh2do --locked
```

Add `~/.cargo/bin` to your PATH if not already:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

## Quick start

Compile a script:
```bash
sh2c -o hello.sh hello.sh2
./hello.sh
```

Run a one-liner:
```bash
sh2do 'print("hello world")'
```

## Documentation

- [Language reference](docs/language.md)
- [sh2do CLI](docs/sh2do.md)

## Issues

Report bugs and request features: https://github.com/siu-mak/sh2lang/issues
