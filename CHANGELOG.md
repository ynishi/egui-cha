# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Workspace crates (`egui-cha`, `egui-cha-ds`, `egui-cha-macros`, `egui-cha-analyzer`)
are released in lock-step and share the same version number.

## [Unreleased]

### Documentation
- Fix installation snippet in README (`egui-cha = "0.1"` → `"0.6"`).
- Add this CHANGELOG.

### Fixed
- Clippy `collapsible_match` warning in `vj-mock` example.

## [0.6.0] - 2026-03-05

### Added
- **egui-cha-ds**: `codedash` metrics visualization components for dashboard-style output.

### Fixed
- Flaky CI tests: replaced `thread::sleep` with `FakeClock`.
- Linux builds: added `x11` / `wayland` features to `eframe`.
- CI: excluded `eframe` examples on Ubuntu, cleared all `cargo clippy --all-targets` warnings.
- Platform-aware feature flags; added `UiFlows` tab usage.

### Changed
- Broad clippy sweep, `#[derive(Default)]` additions, doc-test fixes.
- CI pipeline added (workflow).

## [0.5.0] - 2026-01-26

### Added
- **egui-cha-ds**: `GlassFrame` and `TitleBar` components with window vibrancy support.
- **egui-cha-ds**: TOML-based theme configuration; `LightweightTheme` trait for embedded / no-`serde` scenarios.

## [0.4.0] - 2026-01-25

### Added
- **egui-cha-ds**: `Chat` molecule.

## [0.3.0] - 2026-01-23

### Added
- **egui-cha-ds**: `CommandPalette` molecule.
- **egui-cha-ds**: `QuickActionBar` molecule.
- **egui-cha-ds**: Tooltip delay options.

## [0.2.4] - 2026-01-12

### Added
- **egui-cha-ds**: `DashboardLayout` molecule (`&mut Ui` API).

### Changed
- Consolidated the storybook into `egui-cha-ds` (previously separate).

## [0.2.3] - 2026-01-11

### Changed
- **egui-cha-ds**: Split font setup into `setup_fonts()` so consumers using
  `eframe::run_native()` directly (without `egui_cha::run`) can register the
  Phosphor Icons font on their own.

## [0.2.2] - 2026-01-11

### Added
- **egui-cha-ds**: `LogStream` molecule — real-time log viewer.

### Changed
- **egui-cha-ds**: `Input` atom now honours the active `Theme`.

## [0.2.1] - 2026-01-11

### Added
- **egui-cha-ds**: `CapacityGauge` atom for resource utilization display.

### Changed
- Added `readme` field to every crate for crates.io rendering.

## [0.2.0] - 2026-01-11

### Added
- **egui-cha-ds**: Swarm visualization components.

### Changed
- Enabled crates.io publish for all release targets.
- `vj-mock` example marked `publish = false` and switched to workspace inheritance.

## [0.1.1] - 2025-12-22

### Added
- **egui-cha-ds**: `NodeLayout` infinite-canvas component with resize / collapse /
  maximize / lock (`LockLevel::None|Light|Full`), position-based auto-arrange sort,
  menu bar Lock / Zoom controls.
- **egui-cha-ds**: `WorkspaceCanvas` + drag-to-reorder for `EffectRack` and `LayerStack`.
- **egui-cha-ds**: `arrange_tile` layout; workspace rendering improvements
  (tile reorder + weight-based resize).
- **egui-cha-ds**: `egui-snarl` wrapper with theme integration; NodeGraph demo in storybook.

### Fixed
- **vj-mock**: Timeline width and `ColorWheel` / `OutputRouter` overlap.
- **vj-mock**: Lab-panel spacing.
- **storybook**: NodeGraph demo pin styling.

### Changed
- **egui-cha**: Moved the Phosphor Icons font into the crate-local `assets/`
  directory (packaged with the crate).
- Added `release-patch` / `release-minor` / `release-major` Makefile targets.
- Added crates.io metadata in preparation for release.

## [0.1.0] - 2025-12-21

Initial release. TEA (The Elm Architecture) framework for egui, plus a Design
System crate covering Atoms (Button, Input, Icon, ...), Molecules
(Card, Modal, Tabs, Navbar, ErrorConsole, Toast, ...), a Router, and testing
utilities.

[Unreleased]: https://github.com/ynishi/egui-cha/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/ynishi/egui-cha/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/ynishi/egui-cha/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/ynishi/egui-cha/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/ynishi/egui-cha/compare/v0.2.4...v0.3.0
[0.2.4]: https://github.com/ynishi/egui-cha/compare/v0.2.3...v0.2.4
[0.2.3]: https://github.com/ynishi/egui-cha/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/ynishi/egui-cha/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/ynishi/egui-cha/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ynishi/egui-cha/compare/v0.1.1...v0.2.0
[0.1.1]: https://github.com/ynishi/egui-cha/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/ynishi/egui-cha/releases/tag/v0.1.0
