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

For AppImage bundling, `FUSE` must be available (or set `APPIMAGE_EXTRACT_AND_RUN=1`).

### macOS

Xcode Command Line Tools:

```bash
xcode-select --install
```

## Local Development

Install npm dependencies:

```bash
npm ci
```

Run in development mode (hot reload for frontend, Rust recompilation on save):

```bash
cargo tauri dev --features test-api
```

This opens a window at `http://localhost:1420` with the test API enabled for synthetic data injection.

To run just the frontend dev server (without the Tauri shell):

```bash
npm run dev
```

## Release Build

### Binary only

```bash
cargo tauri build
```

The release binary will be at `src-tauri/target/release/nook`.

### AppImage (Linux)

```bash
cargo tauri build --bundles appimage
```

Output: `src-tauri/target/release/bundle/appimage/nook_<version>_amd64.AppImage`

If `linuxdeploy` fails with a FUSE error, try:

```bash
APPIMAGE_EXTRACT_AND_RUN=1 cargo tauri build --bundles appimage
```

### DMG (macOS)

```bash
cargo tauri build --bundles dmg
```

Output: `src-tauri/target/release/bundle/dmg/Nook_<version>_aarch64.dmg`

## Linting and Type Checking

```bash
npm run lint          # ESLint (Svelte + TypeScript)
npm run check         # svelte-check (type checking)
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

## Tests

```bash
# Rust unit tests
cargo test --manifest-path src-tauri/Cargo.toml

# Visual regression tests (requires test-api build running)
npx playwright test --config tests/visual/playwright.config.ts

# E2E tests (requires Docker running)
npx playwright test --config tests/e2e/playwright.config.ts
```
