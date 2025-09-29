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
cargo build
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

  ```json
  {
    "Grouped": [
      {
        "group": "Other",
        "events": [
          {
            "id": "9aecfd3d-56ab-5cbe-9c4c-158f9e93ce19",
            "timestamp": "2025-09-28T23:30:32Z",
            "endTimestamp": "2025-09-28T23:31:01Z",
            "duration": 29,
            "category": "Other",
            "app": "iTerm2",
            "entity": "esbuild",
            "entityType": "App",
            "project": "iterm2",
            "branch": null,
            "language": null,
            "source": "skopio-desktop"
          }
        ]
      },
      {
        "group": "Browsing",
        "events": [
          {
            "id": "135f5af8-7053-531d-8ab0-2893aa6d419c",
            "timestamp": "2025-09-28T23:30:27Z",
            "endTimestamp": "2025-09-28T23:30:32Z",
            "duration": 4,
            "category": "Browsing",
            "app": "Google Chrome",
            "entity": "/reference/react/useDeferredValue",
            "entityType": "Url",
            "project": "react.dev",
            "branch": null,
            "language": null,
            "source": "skopio-desktop"
          }
        ]
      }
    ]
  }
  ```

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

  ```json
  [
    {
      "id": "019978f0-71fb-7ba0-a472-07f7cd357ac6",
      "name": "Google Chrome"
    },
    {
      "id": "019978f3-311f-7941-a257-7bfa1252728b",
      "name": "iTerm2"
    },
    {
      "id": "019978f3-3123-7453-b1b6-7fa7d37d19a9",
      "name": "Activity Monitor"
    },
    {
      "id": "019987a8-d8fb-73f3-ad9d-efa0de782e68",
      "name": "ChatGPT"
    },
    {
      "id": "019987ed-f3f3-76b2-973f-22ad28da2e4e",
      "name": "Xcode"
    }
  ]
  ```

---

- GET `/projects`

  List tracked projects when supplied with a `ProjectListQuery`

  ```json
  // GET= /projects?after=1758926455%3A0199881f-4613-7ac3-a5bb-2cd7ff27ff5a&limit=15
  {
    "data": [
      {
        "id": "0199881f-4612-7860-b870-c438da91de9f",
        "name": "mozilla.github.io",
        "rootPath": "https://mozilla.github.io/pdf.js/",
        "lastUpdated": 1758925374
      },
      {
        "id": "0199881f-4615-77d1-9e8e-5008403bab9e",
        "name": "github.com",
        "rootPath": "https://github.com/Brendonovich/swift-rs",
        "lastUpdated": 1758925374
      },
      {
        "id": "019978f5-f055-72b3-a8c2-d4fb35cc5515",
        "name": "en.wikipedia.org",
        "rootPath": "https://en.wikipedia.org/wiki/Kris_Wu",
        "lastUpdated": 1758671547
      },
      {
        "id": "019978f3-3124-7522-ba7d-082e3ae2fd95",
        "name": "activity monitor",
        "rootPath": "/System/Applications/Utilities/Activity Monitor.app/Contents/MacOS/Activity Monitor",
        "lastUpdated": 1758671187
      }
    ],
    "totalPages": 2,
    "cursors": [
      "1759103925:019978f3-3120-79d3-8a0d-dc00d683437c",
      "1758926455:0199881f-4613-7ac3-a5bb-2cd7ff27ff5a"
    ]
  }
  ```

---

- GET `projects/:id`

  Search for a project by name when supplied with a `ProjectListQuery`

  ```json
  // GET=/projects?limit=15&query=P
  {
    "data": [
      {
        "id": "0199881f-4613-7ac3-a5bb-2cd7ff27ff5a",
        "name": "pdf-lib.js.org",
        "rootPath": "https://pdf-lib.js.org/",
        "lastUpdated": 1758926455
      }
    ],
    "totalPages": null,
    "cursors": []
  }
  ```

---

- GET `/categories`

  List tracked categories

  ```json
  [
    {
      "id": "019978f0-71fe-7f22-becf-8801a899c1e6",
      "name": "Browsing"
    },
    {
      "id": "019978f3-3121-7db3-982d-9fc28b1dd833",
      "name": "Other"
    },
    {
      "id": "019987ed-f3f7-7081-b8cd-fc9a29871ee2",
      "name": "Coding"
    },
    {
      "id": "01998819-c7d3-7c20-853f-e730c4ee70cf",
      "name": "Writing Docs"
    },
    {
      "id": "0199881f-4615-77d1-9e8e-502a89e72762",
      "name": "Code Reviewing"
    }
  ]
  ```

---

- GET `/summary/total`

  Fetch the total active time when supplied with a `SummaryQueryInput`

  ```bash
  # GET=/summary/total?start=2025-09-28T21%3A00%3A00Z&end=2025-09-29T20%3A59%3A59.999999999Z

  1517
  ```

---

- GET `/summary/buckets`

  Fetch bucketed data grouped by category, app, entity, project, branch, etc., when supplied with a `BucketSummaryInput`

  ```json
  [
    {
      "bucket": "2025-09-24",
      "groupedValues": {
        "Total": 1067
      },
      "groupMeta": null
    },
    {
      "bucket": "2025-09-27",
      "groupedValues": {
        "Total": 1025
      },
      "groupMeta": null
    },
    {
      "bucket": "2025-09-28",
      "groupedValues": {
        "Total": 4959
      },
      "groupMeta": null
    },
    {
      "bucket": "2025-09-29",
      "groupedValues": {
        "Total": 961
      },
      "groupMeta": null
    }
  ]
  ```

---

- GET `/summary/range`

  Fetch the total active time for a particular group when supplied with a `SummaryQueryInput`

---

- GET `/insights`

  Fetch time based insights when supplied with an `InsightQueryPayload`

  ```json
  // GET=/insights?insightType=topN&insightRange=2025&groupBy=language&limit=3
  {
    "topN":[
      ["Swift",767],
      ["Rust",117],
      ["Markdown",28]
    ]
  }
  // GET=/insights?insightType=mostActiveDay&insightRange=2025&bucket=year
  {
    "mostActiveDay":{
      "date":"2025-09-28",
      "total_duration":5503
    }
  }
  ```

---
