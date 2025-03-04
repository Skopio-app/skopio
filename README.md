# Skopio

Skopio is a productivity tool designed to help users track their general app usage and coding stats efficiently. By
leveraging accessibility tooling and editor plugins, Skopio aims to provide insights into coding activity across
multiple platforms. Whether you are coding in an IDE or using other apps, Skopio gathers and presents data in a
meaningful way.

> “You can't improve what you don't measure.” - Peter Drucker.

## Inspiration

The name **skopio** is derived from the Greek word _σκοπέω_ (skopéō), meaning "to observe" or "to examine". It reflects
the tool's purpose of providing insights into a developer's workflow by observing and analyzing coding activity and app
usage.

> [!NOTE]  
> Skopio is still a work in progress.

## Features

- **App Usage Tracking**: Monitors the time spent on different applications.
- **Coding stats**: Tracks coding time across projects, languages, files, editors, branches, etc.
- **Editor support**: Interfaces with popular editors via plugins to collect accurate coding stats.
- **Local-First Approach**: Ensures privacy by keeping data stored locally.
- **Intuitive visualizations**: A lightweight Tauir-based desktop app provides clear summaries and visual insights.

## Project structure

Skopio is a workspace composed of multiple apps working together:

1. **Server app**: A Rust server responsible for handling HTTP requests from editor plugins for data synchronization and
   storage.
2. **CLI app**: A Rust helper command line interface to be used in conjunction with editor plugins for storing raw stats
   and syncing with the server.
3. **Desktop app**: A Rust powered app that aims to provide a lightweight and efficient UI for viewing and tracking
   stats and insights.

## Installation

### Prerequisites

Ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/en)
- [Xcode](https://developer.apple.com/xcode/)

## Usage

- The CLI app integrates with editor plugins to capture coding stats.
- The server processes and stores data from editor plugins.
- The desktop app visualizes tracked usage statistics, as well as capturing general app activity and AFK (Away From
  keyboard) stats via accessibility APIs.

## License

Skopio is licensed under the [MIT License](./LICENSE).
