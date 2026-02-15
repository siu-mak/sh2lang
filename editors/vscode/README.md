# sh2 for Visual Studio Code

<img src="images/sh2logo_256.png" alt="sh2 logo" width="128" />

Syntax highlighting for the **sh2** structured shell language.
> **Versioning Note**: Versions ending in `-dev` (e.g. `0.1.2-dev`) are for local development and testing. For VS Code Marketplace publishing, a standard semantic version (X.Y.Z) is required.

## Manual installation (no Marketplace)

Since sh2 is not yet on the VS Code Marketplace, you must install it manually using one of the methods below.

### Option A: Install from local .vsix (recommended)

**Prerequisites:** Node.js 20+ and npm.

1.  **Build the package:**
    ```bash
    cd editors/vscode
    npm install
    npx @vscode/vsce package
    ```

2.  **Install:**
    -   **VS Code UI:** Open the Extensions view (`Ctrl+Shift+X`), click `...` -> `Install from VSIX...`, and select the generated `.vsix` file.
    -   **CLI:** `code --install-extension sh2-*.vsix`

> **Upgrade note:** To upgrade, build the new `.vsix`, install it again, and reload the window.

### Option B: Install from source (development mode)

This method creates a symlink/junction from your VS Code extensions folder to this directory. Useful for development or if you don't want to build a `.vsix`.

**Paths:**
-   **Linux/macOS:** `~/.vscode/extensions/`
-   **Windows:** `%USERPROFILE%\.vscode\extensions\`

**Instructions:**

1.  Navigate to the `editors/vscode` directory.
2.  Create the link:

    **Linux/macOS:**
    ```bash
    ln -s "$(pwd)" ~/.vscode/extensions/siu-mak.sh2
    ```

    **Windows PowerShell:**
    ```powershell
    New-Item -ItemType Junction -Path "$env:USERPROFILE\.vscode\extensions\siu-mak.sh2" -Target (Get-Location)
    ```

3.  Restart VS Code or run "**Developer: Reload Window**".

### Verify installation

Open a `.sh2` file. Syntax highlighting should activate automatically.

### Troubleshooting

-   **`npx: command not found`**: Install [Node.js](https://nodejs.org/).
-   **`vsce not found`**: Use `npx @vscode/vsce package` or install it globally with `npm install -g @vscode/vsce`.
-   **Extension not loading**:
    -   Run "**Developer: Reload Window**".
    -   Verify the symlink/junction points to the correct directory.
    -   Ensure the file has the `.sh2` extension.

## Features

- Syntax highlighting for `.sh2` files
- Bracket matching and auto-close

- Comment toggling

## Language Features

### Strict Variable Semantics
- Variables must be declared with `let` before use.
- Updates to existing variables use `set`.
- Shadowing is only allowed in disjoint branches (e.g. `if`/`else`).

### Pipeline Iteration
- `each_line` loop is supported (Bash-only) for safe pipeline iteration.
- **Constraint**: `each_line` must be the **last** segment of a pipeline.

> **Note**: sh2 is strict about string literals and paths. It does **not** perform implicit tilde expansion (`~`), globbing (`*`), or variable/parameter expansion (`$name`, `${name}`). See [Strings (`docs/language.md`)](../../docs/language.md#31-strings) for details.

## About sh2

sh2 is a structured shell language that compiles to bash or POSIX sh.
See the [sh2lang repository](https://github.com/siu-mak/sh2lang) for:
- Language documentation
- The `sh2c` compiler
- The `sh2do` snippet runner

## License

Apache-2.0. See [LICENSE](LICENSE).
