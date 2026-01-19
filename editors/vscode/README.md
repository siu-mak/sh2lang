# sh2 VS Code Extension

Syntax highlighting and editor support for the sh2 shell scripting language.

## Features

- Syntax highlighting for `.sh2` files
- Keywords, builtins, operators, strings, numbers, comments
- Bracket matching and auto-closing
- Comment toggling with `Ctrl+/` (uses `#`)

## Installation

### From VSIX (Local)

1. Build the VSIX package (see below)
2. Open VS Code → Extensions → `...` menu → "Install from VSIX..."
3. Select the `sh2-<version>.vsix` file

### From Source (Development)

1. Clone the repo
2. Open `editors/vscode/` in VS Code
3. Press F5 to launch Extension Development Host

## Packaging

### Prerequisites

- Node.js >= 20 (recommended via nvm)
- npm

### Build VSIX

```bash
cd editors/vscode
npm install
npx @vscode/vsce package
```

This creates `sh2-<version>.vsix`.

## Versioning Policy

The extension version **must match** the sh2c compiler version.

| sh2c version | Extension version |
|--------------|-------------------|
| 0.1.0 | 0.1.0 |

This is enforced by `cargo test -p sh2c --test editor_vscode_regression`.

## Release Checklist

1. Bump version in:
   - `sh2c/Cargo.toml`
   - `editors/vscode/package.json`
2. Run guardrail test: `cargo test -p sh2c --test editor_vscode_regression`
3. Package: `npx @vscode/vsce package`
4. Attach `.vsix` to GitHub Release

## Maintenance

When adding new keywords/builtins to sh2:

1. Update `docs/editor_keywords.md`
2. Update `syntaxes/sh2.tmLanguage.json`
3. Run `cargo test -p sh2c --test editor_vscode_regression`
