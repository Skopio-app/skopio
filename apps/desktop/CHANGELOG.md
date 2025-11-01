<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to the desktop app will be documented in this file.

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

## [v0.1.7] -2025-11-02

### Added

- Zen, Helium, Comet, and ChatGPT Atlas are now included in browser list.

### Fixed

- Dashboard widget resize handles now adhere to dark mode.
- Adjusted frames in dino loading screen for smoother animation.
- Project details page bredcrumb back navigation does not display brief white screen.
- AFK state handling in event tracker.
- Added overlap aware logic to event summary functionality.

---

## [v0.1.6] -2025-10-19

### Added

- Server readiness check and loading UI to avoid displaying an empty dashboard.
- Stale data check to reload data display when window resumes focus.

### Fixed

- Branch selection dialog dark mode styling.

---

## [v0.1.5] -2025-10-11

### Fixed

- Update various components' dark mode styling.
- Prevent Xcode and browsers from saving empty projects and entities.
- Detect debugging and compiling states in Xcode on a best-effort basis.
- Handle non-project files being opened in Xcode.

### Added

- General tracking support for most browser apps.
- More VSCode variants are accounted for in ignore list.

### Removed

- Applescript logic for tracking xcode and browser activity.

---

## [v0.1.4] - 2025-10-04

### Changed

- Return a longer entity path when parsing data.
- Switch reload shortcut from `Command + Shift + R` to `Command + R`.

### Added

- New appearance setting to switch app appearance.

---

## [v0.1.3] - 2025-09-30

### Fixed

- Remove redundant inspect element option.

---

## [v0.1.2] - 2025-09-20

### Fixed

- Revert traffic lights position.
- Adjust search icon position in search bar.
- Fetch open apps every time we select the open apps input.
- Server version check now uses semver version.
- AFK timeout option reflects stored value in the filesystem even after reload.

### Added

- Tooltip in chart widgets to give more info on the data displayed.

---

## [v0.1.1] - 2025-09-17

### Fixed

- EOF error when syncing events.

### Changed

- Move traffic light position in UI due to macos Tahoe.
- Modify olling log appender config to save logs in a more accessible manner.
- Add Windsurf to list of ignored apps.
- `File` entity displays an additional path segment.

---

## [v0.1.0] - 2025-09-13

- 🎉 First tagged release of Skopio Desktop.
