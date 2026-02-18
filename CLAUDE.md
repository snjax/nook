# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nook is a cross-platform desktop application for managing dev containers (Linux AppImage, macOS DMG). Built with **Tauri v2** (Rust backend) + **Svelte 5** (frontend, no SvelteKit) + **Vite**.

The authoritative specification is `PRD.md` — consult it for feature details, UI specs, and architectural decisions.

## Build & Development Commands

```bash
# Development (launches Tauri dev server with hot reload + test API)
cargo tauri dev --features test-api

# Build production bundle
NO_STRIP=true ARCH=x86_64 cargo tauri build --bundles appimage  # Linux
cargo tauri build --bundles dmg         # macOS

# Frontend only
npm run dev          # Vite dev server
npm run build        # Production build
npm run lint         # ESLint (Svelte/TS)
npm run check        # svelte-check (type checking)

# Rust backend
cargo test                              # Unit tests
cargo clippy                            # Lint
cargo test --manifest-path src-tauri/Cargo.toml  # Run from repo root

# E2E & Visual regression tests (Playwright)
npx playwright test                     # Visual regression (synthetic data)
npx playwright test e2e                 # E2E tests (requires Docker running)
```

## Architecture

### Rust Backend (`src-tauri/src/`)

- **`commands.rs`** — Tauri command bridge (Rust ↔ Svelte via `invoke()`)
- **`docker/`** — Docker management via `bollard` crate (async): container lifecycle, stats streaming, port detection, process listing
- **`devcontainer/`** — Wraps `devcontainer` CLI (`tokio::process::Command`) for `up/exec/stop` and `.devcontainer/` directory scanning
- **`terminal/`** — Detects system terminal and default shell; launches terminal sessions into containers
- **`network/`** — 3-stage port detection pipeline:
  1. Light polling (`ss -tlnp` inside container, every 2-3s)
  2. Fast heuristics (well-known ports + process name matching)
  3. Targeted `nmap -sV` + banner-grab (only for unknown ports)
- **`config/`** — Settings and pod state persistence (TOML files in `~/.config/nook/`)

### Svelte Frontend (`src/`)

- **`lib/components/`** — UI components (PodTile, PortBadge, PortPrompt, ResourceChart, etc.)
- **`lib/stores/`** — Svelte stores for pods, ports, settings state
- **`lib/api/`** — Typed wrappers around Tauri `invoke()` calls; separate `test-api.ts` for dev mode

### Communication Pattern

Frontend calls Rust via Tauri commands (`invoke()`). Rust pushes real-time updates (stats, port detection, process lists) via Tauri events that frontend subscribes to.

### Key Domain Concepts

- **Pod** = a dev container with lifecycle state machine: `Stopped → Starting → Running → Stopping → Stopped` (with `Error` transitions)
- **Port policies** = per-port expose/ignore decisions persisted across restarts
- **Test API** = feature-gated (`#[cfg(feature = "test-api")]`) synthetic data injection for Playwright visual tests

## Testing

Three test levels, all mandatory:

1. **Rust unit tests** (`cargo test`) — parsing, heuristics, config serialization
2. **Visual regression** (Playwright, `tests/visual/`) — snapshot comparison across viewports (1280×720 → 3840×2160), uses test API for synthetic data, `maxDiffPixelRatio: 0.001`
3. **E2E** (Playwright, `tests/e2e/`) — real Docker lifecycle with a Node.js/Express devcontainer fixture; timeouts: 180s first build, 60s cached, 15s port detection

## Key Dependencies

**Rust**: `tauri`, `tokio`, `bollard`, `serde`, `toml`, `dirs`, `regex`
**Node**: `svelte`, `vite`, `lucide-svelte`, `chart.js`/`svelte-chartjs`, `@playwright/test`

## Platform Considerations

- Docker socket: `/var/run/docker.sock` (Linux) vs `/var/run/docker.sock` or `~/.docker/run/docker.sock` (macOS)
- Terminal detection: `$TERMINAL` → `x-terminal-emulator` → known terminals (Linux); `osascript` for Terminal.app/iTerm2 (macOS)
- Config path: `~/.config/nook/` (Linux), `~/Library/Application Support/nook/` (macOS)
- WebKitGTK CSS limitations on Linux (no backdrop-filter, limited animations)
- AppImage build on Arch/newer distros: `NO_STRIP=true ARCH=x86_64 cargo tauri build --bundles appimage` — the bundled `linuxdeploy` has an old `strip` that fails on `.relr.dyn` sections, and `appimagetool` needs explicit arch when GTK plugin bundles mixed-arch libs

## Development Rules

- **Always verify after changes**: after making edits, run the relevant tests and build to confirm correctness. If tests or build fail — fix the errors before moving on.
- **Test coverage is mandatory**: any feature or fix that can be covered by tests must be covered. Add unit tests (Rust), visual regression, or E2E tests as appropriate.
- **Always check library docs via MCP**: before using any library API, look up its documentation in Context7. If Context7 is unavailable, use Perplexity. Never rely on model memory for API details — versions and interfaces change.
- All interactive UI elements must have `data-testid` (Playwright) + `aria-label`
- Use mutex per pod ID to prevent race conditions on concurrent container actions
- Errors: inline in pod tiles for per-pod issues, global modal only for app-level failures — no popup spam
- Flat UI style (borders, no shadows), dark theme with VS Code color palette + Docker blue accent (`#0098ff`)

## Self-Improvement

If during development you discover an insight, pattern, gotcha, or workaround that would make future work in this repo more efficient — **add it to this CLAUDE.md immediately**. This file is your persistent memory across sessions. Examples of what to record: non-obvious build quirks, platform-specific pitfalls, library API surprises, architectural decisions that weren't obvious from the code alone. Keep entries concise and actionable.
