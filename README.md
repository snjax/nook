# Nook

Cross-platform desktop application for managing dev containers.
Built with **Tauri v2** (Rust backend) + **Svelte 5** (frontend) + **Vite**.

Target platforms: Linux (AppImage), macOS (DMG).

## Prerequisites

### All platforms

- **Node.js** >= 22
- **Rust** (stable toolchain) — install via [rustup](https://rustup.rs/)
- **Tauri CLI** v2:
  ```bash
  cargo install tauri-cli --version "^2"
  ```
- **just** — command runner ([github.com/casey/just](https://github.com/casey/just))
- **Docker** — running daemon with access to `/var/run/docker.sock`
- **devcontainer CLI** (optional, for dev container lifecycle):
  ```bash
  npm install -g @devcontainers/cli
  ```

### Linux (Ubuntu/Debian)

System libraries required by WebKitGTK:

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libgtk-3-dev
```

### macOS

Xcode Command Line Tools:

```bash
xcode-select --install
```

## Quick Start

```bash
just setup   # install npm + cargo dependencies
just dev     # launch Tauri dev server with hot reload
```

## Commands

All commands are defined in the `Justfile`. Run `just --list` to see them all.

| Command | Description |
|---------|-------------|
| `just dev` | Tauri dev server with hot reload + test API |
| `just dev-frontend` | Vite dev server only (no Tauri shell) |
| `just build` | AppImage release build (Linux) |
| `just build-dmg` | DMG release build (macOS) |
| `just build-binary` | Release binary only |
| `just test` | Run all tests (Rust + frontend type check) |
| `just test-rust` | Rust unit tests |
| `just test-visual` | Playwright visual regression tests |
| `just test-e2e` | Playwright E2E tests (requires Docker) |
| `just lint` | Lint Rust (clippy) + frontend (ESLint) |
| `just check` | Type check Rust + Svelte |
| `just ci` | Full CI pipeline: lint + check + test + build |
| `just setup` | Install all dependencies |
| `just icon <path>` | Generate app icons from a 1024x1024 PNG |

## Build Outputs

- **Linux**: `src-tauri/target/release/bundle/appimage/Nook_<version>_amd64.AppImage`
- **macOS**: `src-tauri/target/release/bundle/dmg/Nook_<version>_aarch64.dmg`
- **Binary**: `src-tauri/target/release/nook`
