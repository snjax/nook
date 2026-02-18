# Nook — dev container manager

# Development
dev:
    cargo tauri dev --features test-api

dev-frontend:
    npm run dev

# Build
build:
    NO_STRIP=true ARCH=x86_64 cargo tauri build --bundles appimage

build-dmg:
    cargo tauri build --bundles dmg

build-binary:
    cargo tauri build

build-frontend:
    npm run build

# Lint
lint: lint-rust lint-frontend

lint-rust:
    cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

lint-frontend:
    npm run lint

# Type check
check: check-rust check-frontend

check-rust:
    cargo check --manifest-path src-tauri/Cargo.toml

check-frontend:
    npm run check

# Tests
test: test-rust test-frontend

test-rust:
    cargo test --manifest-path src-tauri/Cargo.toml

test-frontend:
    npm run check

test-visual:
    npx playwright test --config tests/visual/playwright.config.ts

test-e2e:
    npx playwright test --config tests/e2e/playwright.config.ts

# Run all checks (lint + test + build)
ci: lint check test build-frontend
    @echo "All checks passed"

# Setup
setup:
    npm ci
    cargo fetch --manifest-path src-tauri/Cargo.toml

# Generate app icons from a 1024x1024 PNG
icon path:
    cargo tauri icon {{path}}
