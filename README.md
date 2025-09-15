# Skopio

Skopio is a productivity tool designed to help users track their general app usage and coding stats efficiently. By
leveraging accessibility tooling and editor plugins, Skopio aims to provide insights into coding activity across
multiple platforms. Whether you are coding in an IDE or using other apps, Skopio gathers and presents data in a
meaningful way.

> “You can't improve what you don't measure.” - Peter Drucker.

## Inspiration

The name **skopio** is derived from the Greek word _σκοπέω_ (skopéō), meaning "to observe" or "to examine". It reflects
the tool's purpose of providing insights into a user's workflow by aggregating and presenting coding activity and general app
usage.

> [!NOTE]
> Skopio is still a work in progress.

## Features

- **App Usage Tracking**: Monitors the time spent in different applications.
- **Coding stats**: Tracks coding time across projects, languages, files, editors, branches, etc.
- **Editor support**: Interfaces with popular editors via plugins/extensions to collect accurate coding stats.
- **Local-First Approach**: Ensures privacy by keeping data stored locally.
- **Intuitive visualizations**: A lightweight Tauri-based desktop app provides clear summaries and visual insights.

## Project structure

Skopio is a workspace composed of multiple apps working together:

1. [**Server app**](./apps/server/README.md): A Rust server app responsible for handling HTTP requests from editor plugins and the desktop app for data synchronization and
   storage.
2. [**CLI app**](./apps/cli): A Rust helper command line interface to be used in conjunction with editor plugins for storing raw stats
   and syncing with the server.
3. [**Desktop app**](./apps/desktop/README.md): A tauri app that aims to provide a lightweight and efficient UI for viewing and tracking
   stats and insights.

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](./CONTRIBUTING.md) for guidelines.

## License

Skopio is licensed under the [MIT License](./LICENSE).
