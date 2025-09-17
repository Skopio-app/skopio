# Contributing

If you are interested in contributing to the project, please read the following guidelines.

## Development

### Prerequisites

- [Node.js](https://nodejs.org/en)
- [Yarn](https://yarnpkg.com/)
- [Rust](https://www.rust-lang.org/)

### Setup

```bash
git clone https://github.com/Skopio-app/skopio --recursive
yarn install
```

Given that we use pre-commit hooks for running tasks such as code formatting and linting, run the following commands to
set up the pre-commit config already in the project:

```bash
pip install pre-commit
pre-commit install
```

For any changes to the database schema over at `crates/db/migrations`, make sure to run `./scripts/sync_db.sh`.

In order to guarantee that both desktop and server query validations are satisfied whenever you make changes to queries in the `db` crate, make sure to run `./scripts/merge_sqlx.sh`.

### Run Desktop App

```bash
yarn workspace @skopio/desktop tauri dev
# or run it within the desktop app directory
cd apps/desktop
yarn tauri dev
```

### Run server app

The desktop app communicates with the server, therefore to view the gathered insights and data, the server has to be running too.

```bash
cd apps/server
cargo build
cargo run
```

### Run CLI app

The CLI is a Rust-based helper tha sits between your **editor plugins/extensions** (VSCode, Jetbrains, Zed, etc.) and the server.

It captures coding activity from editors, saves it to a SQLite database, and peridiocally syncs the saved data to the skopio server to be aggregated then presented via the desktop app.

```bash
cd apps/cli
cargo build
cargo run -- --help
```
