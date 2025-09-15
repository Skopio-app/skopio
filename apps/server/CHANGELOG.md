<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to the server app will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html)

## [Unreleased]

### Added

- (placeholder)

### Changed

- (placeholder)

### Fixed

- (placeholder)

---

## [v0.1.0] - 2025-09-11

### Added

- Initial release of the Skopio Server.
- Axum-based API with routes as described in the [README](./README.md)
- Support for both **Unix domain sockets** (prod) and **TCP** for dev.
- Integration with the desktop app and CLI for secure ingestion and querying.
- Local SQLite storage using `sqlx`.
