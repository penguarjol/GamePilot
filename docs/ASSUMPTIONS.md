# GamePilot MVP Assumptions

Decisions made where the PRD/ADR were ambiguous or silent.

## A1: Launch delegation over direct Java launch
The MVP delegates launch to the detected launcher (Prism, CurseForge, etc.)
rather than constructing a full Java command line. Direct Java launch is
complex (classpath, natives, auth) and fragile. If no launcher is detected,
the app opens the instance folder and instructs the user to launch manually.

## A2: Java detection — detect only, no install/download
MVP detects installed Java runtimes by scanning standard paths and PATH.
It does not download, install, or bundle a Java runtime. Recommendations
point the user to download links.

## A3: FPS telemetry deferred to post-MVP
FPS/frame-time capture requires PresentMon/ETW or game-specific integration.
MVP ships CPU/RAM/process/session reporting. FPS is documented as deferred.

## A4: No tray app or background daemon for MVP
GamePilot runs as a foreground desktop app. No system tray icon or background
service. The performance governor monitors self-usage only while the app is open.

## A5: Unsigned installer with SmartScreen documentation
No code-signing certificate is available. The Windows installer is unsigned.
Documentation explains expected SmartScreen warnings and how to proceed.

## A6: Mod metadata — curated starter set, not exhaustive
The mod metadata database starts with ~30 well-known performance/utility mods.
Unknown mods are reported honestly as "unclassified." The format supports
future expansion.

## A7: Data retention — 90 days default, user-configurable
Session data and telemetry summaries are retained for 90 days by default.
Users can change this in settings or delete all data.

## A8: Single-user, no accounts
MVP has no user accounts, login, or cloud sync. All data is local to the
device. A single implicit user profile is used.

## A9: Tauri 2 file dialog for manual instance selection
Manual instance/modpack selection uses Tauri's native file dialog API
rather than a custom file browser component.

## A10: SQLite database location
The SQLite database is stored in the Tauri app data directory
(`%APPDATA%/com.gamepilot.app/` on Windows, platform equivalent elsewhere).

## A11: Cross-platform dev on macOS, target Windows
Development uses macOS with platform adapter stubs. Windows-specific code
is verified via GitHub Actions CI on windows-latest. Features that require
real Windows APIs are clearly marked and tested only in CI.

## A12: Rollback scope — JVM/config files only for MVP
The rollback system covers JVM argument changes and config file edits.
Mod installation/removal rollback is deferred to post-MVP.
