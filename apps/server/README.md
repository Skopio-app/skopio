# Skopio Server

The Skopio server is the local backend component of the Skopio ecosystem. It provides a secure, local-first API for storing and querying coding activity, app usage, and AFK stats.

The server runs in the background and communicates with the Skopio desktop app and CLI over a secure channel.

## Features

- Axum-based API
- Local first storage - SQLite using `sqlx`
- Transport - Unix Domain Socket (UDS) in production, TCP in dev.
- Events model to capture
  - Project association
  - Branch
  - Language
  - AFK stats
  - Entity (`File`, `App`, `Url`) data

## Releases

The server is shipped as a standalone binary via the [server-releases](https://github.com/Skopio-app/server-releases) repo. It is automatically downloaded, installed, and managed by the Skopio desktop app.

## Development

```bash
# Run in dev mode
cargo run
```

## API endpoints

- POST /events

  Insert a batch of events

```json
[
  {
    "timestamp": "2025-09-11T12:34:56Z",
    "end_timestamp": "2025-09-11T12:36:56Z",
    "duration": 120,
    "category": "Coding",
    "app_name": "Code",
    "entity_name": "main.rs",
    "project_name": "skopio",
    "branch_name": "main",
    "language_name": "Rust",
    "source_name": "skopio-desktop"
  }
]
```

---

- GET `/events`

  Fetch a batch of events, when provided with a `BucketSummaryInput`

---

- POST `/afk`

  Insert AFK events

```json
[
  {
    "afk_start": "2025-09-11T12:00:00Z",
    "afk_end": "2025-09-11T12:15:00Z",
    "duration": 900
  }
]
```

---

- GET `/apps`

  List saved tracked apps.

---

- GET `/projects`

  List tracked projects

---

- GET `/categories`

  List tracked categories

---

- GET `/summary/total`

  Fetch the total active time when supplied with a `SummaryQueryInput`

---

- GET `/summary/buckets`

  Fetch bucketed data grouped by category, app, entity, project, branch, etc., when supplied with a `BucketSummaryInput`

---

- GET `/summary/range`

  Fetch the total active time for a particular group when supplied with a `SummaryQueryInput`

---

- GET `/insights`

  Fetch time based insights when supplied with an `InsightQueryPayload`

---
