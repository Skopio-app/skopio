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

## [v0.1.9] - 2025-12-20

### Fixed

- Join by event overlap as opposed to start-in-bucket in insight query logic.

---

## [v0.1.8] - 2025-12-20

### Fixed

- Join by event overlap as opposed to start-in-bucket in summary query logic.

---

## [v0.1.7] - 2025-12-01

### Fixed

- Overlap awareness when computing event summaries or total time spent given a range.

---

## [v0.1.6] - 2025-11-29

### Fixed

- Slow summary queries

---

## [v0.1.5] - 2025-11-02

### Fixed

- Add event range overlap aware logic to summary functionality

### Added

- Additional tests for overlap logic and query helpers.

---

## [v0.1.4] - 2025-10-19

### Changed

- Add health status object in `/health` route to enhance server readiness check.

---

## [v0.1.3] - 2025-09-30

### Fixed

- Removed redundant indexes.

### Added

- `last_updated` field to keep track of recently accessed items.

### Changed

- Use integers instead of text when storing timestamp values.
- Rename project routes to follow REST conventions.

### Removed

- Redundant `without_row_id` config.

---

## [v0.1.2] - 2025-09-17

### Fixed

- Add stable id to dedupe duplicate events.

### Added

- Timestamp indexes to speed up queries.

### Changed

- Modify rolling log appender config to save logs in a more accessible manner.

---

## [v0.1.1] - 2025-09-16

### Fixed

- Improved batch insertion of events and AFK events

---

## [v0.1.0] - 2025-09-11

### Added

- Initial release of the Skopio Server.
- Axum-based API with routes as described in the [README](./README.md)
- Support for both **Unix domain sockets** (prod) and **TCP** for dev.
- Integration with the desktop app and CLI for secure ingestion and querying.
- Local SQLite storage using `sqlx`.
