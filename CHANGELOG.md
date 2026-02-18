# Changelog

## 0.1.0 - 2026-02-18

- Added streaming limit enforcement, including MIME allowlist validation.
- Added storage abstraction with `MemoryStorage` and `DiskStorage`.
- Added `DiskStorage` filename strategies: `Keep`, `Random`, and `Custom`.
- Added framework-agnostic end-to-end parse-and-store APIs.
- Added optional Axum and Actix integration helpers behind feature flags.
- Added examples, benchmark scaffold, and CI workflow gates.
