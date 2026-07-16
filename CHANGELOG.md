# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Workspace crates (`egui-cha`, `egui-cha-ds`, `egui-cha-macros`, `egui-cha-analyzer`)
are released in lock-step and share the same version number.

## [Unreleased]

## [0.7.2] - 2026-07-17

### Fixed
- **egui-cha-ds**: annotate the remaining `f32` literals in the storybook
  example (six sites missed by 0.7.1) so the `float_literal_f32_fallback`
  lint no longer breaks CI clippy/test.

## [0.7.1] - 2026-07-16

### Fixed
- **egui-cha-ds**: annotate float literals as `_f32` to satisfy the new
  `f32: From<f64>` type-inference lint (rust-lang/rust#154024), which had
  turned into a hard error on the latest stable toolchain and broke CI.

## [0.7.0] - 2026-07-16

### Added
- **egui-cha-ds**: `WorkspaceCanvas::debug_overlay(bool)` draws a layout
  debug overlay (canvas / available / pane rects with coordinates); the
  storybook gained a Debug toggle for it.

### Fixed
- **egui-cha-ds**: `WorkspaceCanvas` no longer rewinds the parent Ui cursor
  (pane title-bar interactions used `allocate_rect`), which made content
  placed after the canvas flow back into it and overlap the panes.
- **egui-cha-ds**: Free-mode panes are clamped to the canvas, painting is
  clipped to it, and the canvas reserves at least the height of its panes
  inside scroll areas.
- **egui-cha-ds**: minimized `WorkspaceCanvas` panes collapse to their title
  bar and can be restored, instead of disappearing with no way back.
- **egui-cha-ds**: the pane lock icon no longer overlaps the title text, and
  the canvas lock indicator is readable on light themes.
- **egui-cha**: Reactive repaint mode now wakes the UI thread when an async
  task completes or an interval ticks; previously the message sat in the
  channel until the next user input.
- **egui-cha**: idle apps with active intervals no longer repaint at full
  framerate in Reactive mode.
- **egui-cha**: `Cmd::Msg` chains are drained within a single frame (bounded
  at 16 update passes) instead of advancing one message per frame.
- **egui-cha**: tokio runtime creation failure is propagated as an error from
  `run` instead of panicking.
- Clippy `collapsible_match` warning in `vj-mock` example.

### Changed
- **egui-cha-macros**: `cha!` now rejects unknown layout properties (e.g. a
  typo like `Col(spacig: 8.0)`) with a spanned compile error; previously they
  were silently ignored. DSL parse errors carry contextual messages.
- **egui-cha-ds**: `semantics::button`, `Navbar`, `Tabs`, `Toggle`, `Link`,
  and `Modal` now read their colors from `Theme::current` tokens instead of
  hardcoded dark-mode branches, so custom themes propagate. Visual deltas:
  semantic Secondary buttons in dark mode now match `Button::secondary`;
  light-mode Toggle off-track and Modal background shift one shade to the
  nearest token.
- **egui-cha-ds**: `GlassFrame::show` honors `Theme::glass_tint` when the
  frame has no explicit tint (previously only `GlassFrame::from_theme` wired
  it).

### Documentation
- Fix installation snippet in README (`egui-cha = "0.1"` → `"0.6"`).
- README: dual-license section (MIT OR Apache-2.0); component tables marked
  as a selection instead of stale counts.
- Removed stale dock/tiles TODO comments in `egui-cha-ds` molecules.
- Add this CHANGELOG.

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
