# Skopio CLI

`skopio-cli` is the helper process used by Skopio editor plugins to buffer activity events locally and forward them to the main Skopio server.

It has two jobs:

- `event`: store a single activity event in the local CLI database
- `sync`: send all unsynced events to the main Skopio server and mark them as synced

## Usage

```bash
skopio-cli [COMMAND]
```

Top-level commands:

- `event`: Save an event
- `sync`: Sync stored events to the main server
- `help`: Print command help

## `event` command

Stores one activity event in the local CLI database.

```bash
skopio-cli event \
  --timestamp <TIMESTAMP> \
  --category <CATEGORY> \
  --app <APP> \
  --entity <ENTITY> \
  --entity-type <ENTITY_TYPE> \
  --duration <DURATION> \
  --project <PROJECT> \
  --source <SOURCE> \
  --end-timestamp <END_TIMESTAMP>
```

Arguments and flags:

- `-t, --timestamp <TIMESTAMP>`: Start time as a Unix timestamp
- `-c, --category <CATEGORY>`: Activity category such as `Coding` or `Debugging`
- `-a, --app <APP>`: App being tracked
- `-e, --entity <ENTITY>`: Entity path or identifier being tracked
- `--entity-type <ENTITY_TYPE>`: Entity type such as `App`, `File`, or `Url`
- `-d, --duration <DURATION>`: Event duration in seconds
- `-p, --project <PROJECT>`: Full path of the current project
- `-s, --source <SOURCE>`: Plugin or extension that generated the event
- `--end-timestamp <END_TIMESTAMP>`: End time as a Unix timestamp
- `-h, --help`: Print help for the `event` command

Example:

```bash
skopio-cli event \
  --timestamp 1761962400 \
  --category Coding \
  --app "Visual Studio Code" \
  --entity "/Users/samuelwahome/CodeProjects/skopio/apps/cli/src/main.rs" \
  --entity-type File \
  --duration 120 \
  --project "/Users/samuelwahome/CodeProjects/skopio" \
  --source skopio-vscode \
  --end-timestamp 1761962520
```

## `sync` command

Uploads all unsynced events from the local CLI database to the main Skopio server.

```bash
skopio-cli sync
```

Flags:

- `-h, --help`: Print help for the `sync` command

Example:

```bash
skopio-cli sync
```
