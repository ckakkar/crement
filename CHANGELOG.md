# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-05-25

### Added

- `crement!` — unified macro supporting all four C-style forms:
  `++x`, `x++`, `--x`, `x--`.
- `pre_inc!` — prefix increment convenience alias.
- `post_inc!` — postfix increment convenience alias; returns the old value
  via `Clone::clone` (zero-cost for `Copy` types).
- `pre_dec!` — prefix decrement convenience alias.
- `post_dec!` — postfix decrement convenience alias.
- Hygienic internal temporaries via `Span::mixed_site()`.
- `Joint`-spacing check to distinguish `++`/`--` from `+ +`/`- -`.
- 79 integration tests covering all primitive integer types, Clone-only
  types, complex lvalue expressions (field access, array/slice index,
  `Box<T>` deref), sequences, and loop patterns.
- 4 compile-fail trybuild tests confirming helpful diagnostics for missing
  `Clone`, empty input, missing operator, and spaced operator.
- GitHub Actions CI: test matrix (stable / beta / nightly / MSRV 1.65) across
  Ubuntu, macOS, and Windows; Clippy; rustfmt; cargo-audit; publish dry-run.
- GitHub Actions publish workflow: triggers on `v*` tags, verifies tag matches
  `Cargo.toml` version, runs tests, publishes to crates.io, creates GitHub
  release.

[Unreleased]: https://github.com/ckakkar/crement/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ckakkar/crement/releases/tag/v0.1.0
