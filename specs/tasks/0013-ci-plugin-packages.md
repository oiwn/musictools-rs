---
id: 0013-ci-plugin-packages
status: draft
scope: [.github/workflows/ci.yml, .github/workflows/release.yml]
created: 2026-10-29
source: user request 2026-10-29: CI building cross-platform plugin packages
---
# Task: ci plugin packages

## Plan

- [ ] `ci.yml` on push and PR: `cargo fmt --all --check`, Clippy with `-D warnings`, `cargo test --workspace --all-features`, `typos`, `gitleaks git --verbose` — mirroring `prek.toml`.
- [ ] `release.yml`: triggers on tags `v*` plus `workflow_dispatch`; matrix is macos-latest aarch64 (native), macos-latest with the `x86_64-apple-darwin` target, ubuntu-latest x86_64, windows-latest x86_64.
- [ ] Install cargo-truce 6.3.0 (`cargo install truce --version 6.3.0 --locked`); cache with rust-cache.
- [ ] Build each release plugin (`bogdan`, `kirya`; `roman` joins after 0009 lands): `cargo truce build -p <name> --clap --vst3`. Igorek is excluded per the 2026-10-29 decision.
- [ ] Package per target: CLAP and VST3 bundles from `target/bundles` plus a generated INSTALL.md (per-OS drop-in folder paths, macOS Gatekeeper note) as `musictools-<version>-<os>-<arch>` archives; upload as workflow artifacts and attach to a GitHub Release on tags.
- [ ] Green manual-dispatch run with artifacts downloadable.

## Acceptance

- [ ] `cargo fmt --all --check`
- [ ] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [ ] `cargo test --workspace --all-features`
- [ ] `typos`
- [ ] `gitleaks git --verbose`

## Manual checks

- [ ] Download one package from a run, drop the bundles into the host's plugin folders, plugins load.

## Context

- Decisions 2026-10-29: slim 3-OS matrix (macOS aarch64, Linux x86_64, Windows x86_64 — plus macOS x86_64 cross-built on the arm runner); no installer, no code signing.
- No CI exists today (no `.github/`); this task adds both workflows.

## Summary

GitHub Actions: PR checks mirroring the local hooks, plus a release matrix packaging CLAP and VST3 bundles for macOS, Linux, and Windows.

## Log

- 2026-10-29 created: ci plugin packages
- 2026-10-29 set source user request 2026-10-29: CI building cross-platform plugin packages
- 2026-10-29 scope add .github/workflows/ci.yml — two new GitHub Actions workflows
- 2026-10-29 scope add .github/workflows/release.yml — two new GitHub Actions workflows
