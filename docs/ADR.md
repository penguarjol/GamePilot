# GamePilot Architecture Decision Records

**Status:** Proposed initial architecture  
**Date:** 2026-07-08  
**Audience:** Cursor agent and implementation subagents  
**Related:** `docs/PRD.md`

This document captures the initial architecture decisions needed to turn the README conversation into an executable engineering plan. These ADRs favor a Windows-first Minecraft MVP while preserving the long-term platform direction: a universal gaming intelligence platform that can support many games through modular capabilities.

## Architecture Summary

GamePilot should be built as a local-first Windows desktop application with a strict resource budget. The core platform is game-agnostic. Game-specific behavior lives in modules that discover game state, provide rules and knowledge, and request safe actions. The core owns telemetry, recommendation orchestration, optimization execution, backups, rollback, and UI.

```text
Native Desktop UI
  |
Core Platform
  |-- Plugin / Module Manager
  |-- Event Bus
  |-- Hardware Layer
  |-- Process Analyzer
  |-- Telemetry Engine
  |-- Rule Engine
  |-- Recommendation Engine
  |-- Optimization Engine
  |-- Safety / Rollback Manager
  |-- Performance Governor
  |-- Local Data Store
  |-- Knowledge Engine
  |-- Optional AI Runtime (future)
  |
Game Modules
  |-- Minecraft MVP Module
  |-- Future League Module
  |-- Future RuneScape Module
  |-- Future Tarkov Module
```

## ADR-0001: Product Architecture Is a Universal Gaming Intelligence Platform

**Status:** Accepted for MVP direction

### Context

The conversation began with Minecraft server and modpack optimization, then expanded into a broader idea: a game-agnostic app that optimizes performance, understands game state, learns from sessions, and gives personalized recommendations. Calling the product a launcher would understate the architecture and create the wrong boundaries.

### Decision

GamePilot is a universal gaming intelligence platform. Launching games is one capability. Performance optimization is the first specialization. Minecraft is the first implementation module.

The core platform must not contain Minecraft-specific code. The core should understand normalized concepts:

- Game
- Game instance
- Launch profile
- Hardware profile
- Process/resource signal
- Telemetry event
- Game state event
- Recommendation
- Optimization action
- Session
- Outcome

### Consequences

- The MVP can stay focused on Minecraft without hardcoding Minecraft into the platform.
- Future games can add intelligence modules without rewriting core systems.
- The initial data model must be general enough for future game state, not only FPS and JVM settings.
- Product copy and architecture should avoid positioning GamePilot as only a launcher.

## ADR-0002: Windows-First Desktop Stack

**Status:** Proposed

### Context

The app must feel like modern Windows desktop software and must be lightweight. Electron is likely too heavy for the stated performance budget. A pure WinUI/.NET app gives native Windows controls but makes future cross-platform support and Rust-based low-level systems work less direct. Tauri gives a small native shell, Rust backend, Windows integration, strong process/system access, and a modern UI layer.

### Decision

Use:

- **Desktop runtime:** Tauri 2
- **Systems/backend:** Rust
- **UI:** React + TypeScript
- **Build tooling:** Vite
- **Local database:** SQLite
- **Rust data access:** sqlx or rusqlite
- **Process/hardware collection:** Rust crates plus Windows APIs where needed
- **Packaging:** Tauri Windows installer target first

The UI must not look like a generic web app. It should use custom desktop-focused layout, dense information design, keyboard navigation, responsive panes, and native Windows integration where useful.

### Alternatives Considered

- **Electron:** easiest UI ecosystem, but poor fit for strict idle and monitoring memory budgets.
- **WinUI 3/.NET:** strongest native Windows feel, but less aligned with future cross-platform direction and Rust systems modules.
- **Avalonia/.NET:** cross-platform and desktop-native, but smaller ecosystem and less direct fit for high-fidelity web-style UI iteration.
- **Rust native UI only:** potentially efficient but high product/UI iteration cost.

### Consequences

- Tauri/WebView2 overhead must be measured early against the RAM budget.
- The first implementation should include a performance regression harness.
- If Tauri cannot meet the resource budget on realistic hardware, revisit WinUI 3 with Rust sidecars.
- Cursor should scaffold a thin UI and validate baseline resource use before building deep features.

## ADR-0003: Modular Game Capability Model

**Status:** Accepted

### Context

The transcript uses "plugins," "game intelligence modules," and "capabilities." The durable abstraction is not a plugin that can do arbitrary things. It is a game module that implements common platform capabilities.

### Decision

Each game module declares the capabilities it supports. Initial capabilities:

- `discover_game`
- `discover_instances`
- `discover_launch_profiles`
- `provide_state`
- `provide_knowledge`
- `provide_rules`
- `analyze_configuration`
- `recommend_optimizations`
- `launch`
- `summarize_session`

The core defines capability interfaces. Modules provide data and requested actions. The core validates and executes privileged changes.

For MVP, modules can be compiled into the app as Rust crates. A dynamic third-party SDK can come later after the interfaces stabilize.

### Consequences

- Minecraft can be implemented quickly without designing a public SDK too early.
- The core remains responsible for safety.
- Future dynamic plugins, WebAssembly modules, or signed extension bundles can be added without changing product concepts.
- Module boundaries should be tested with mocked core services.

## ADR-0004: Core-Owned Optimization Execution

**Status:** Accepted

### Context

GamePilot may edit configs, change launch arguments, pause processes, or restore settings. Letting game modules directly mutate files and processes would make safety, rollback, and audit logs inconsistent.

### Decision

Game modules cannot directly apply system changes. They emit `OptimizationActionRequest` records. The core optimization engine validates, previews, backs up, applies, verifies, logs, and rolls back actions.

Initial action types:

- `EditFileWithBackup`
- `SetLaunchArgument`
- `SetEnvironmentVariable`
- `SelectJavaRuntime`
- `RecommendProcessClose`
- `OpenExternalLink`
- `CreateReport`
- `NoOpRecommendation`

Deferred action types:

- `InstallMod`
- `RemoveMod`
- `ReplaceMod`
- `DownloadRuntime`
- `PauseService`
- `ApplyGraphicsSetting`

### Consequences

- Rollback behavior is consistent across games.
- The UI can show one preview/apply/rollback flow for all optimization actions.
- Modules are easier to trust and test.
- Some module code may feel indirect, but the safety model is more important.

## ADR-0005: Recommendation Contract

**Status:** Accepted

### Context

The user wants explainable, measurable, reversible recommendations. Generic suggestions like "optimize settings" are not actionable enough.

### Decision

Every recommendation must have a normalized schema:

```text
Recommendation
  id
  game_id
  instance_id
  session_id optional
  category
  severity
  title
  description
  evidence[]
  expected_impact
  confidence
  risk
  action_request optional
  rollback_strategy optional
  status
  created_at
```

Confidence values:

- `low`
- `medium`
- `high`

Risk values:

- `none`
- `low`
- `medium`
- `high`

Recommendation statuses:

- `new`
- `accepted`
- `applied`
- `ignored_once`
- `ignored_always`
- `deferred`
- `rolled_back`
- `failed`

### Consequences

- UI can render every recommendation consistently.
- Recommendation history can improve future confidence.
- The system can avoid false precision while still giving useful impact estimates.
- Cursor should implement the schema early because many features depend on it.

## ADR-0006: Event Bus as the Internal Backbone

**Status:** Accepted

### Context

The platform needs game lifecycle events, telemetry events, module events, recommendation events, and safety/rollback events. Tight coupling between systems would make the app brittle.

### Decision

Implement an internal event bus with typed events. Events are persisted selectively. High-volume telemetry samples should be summarized before persistence.

Initial event families:

- App lifecycle
- Discovery
- Game launch/session
- Hardware/process telemetry
- Minecraft analysis
- Recommendation
- Optimization action
- Safety/rollback
- Performance governor

Example events:

- `GameDiscovered`
- `MinecraftInstanceDiscovered`
- `GameLaunchRequested`
- `GameLaunched`
- `GameExited`
- `CpuPressureDetected`
- `MemoryPressureDetected`
- `BackgroundProcessRiskDetected`
- `RecommendationCreated`
- `OptimizationApplied`
- `OptimizationRolledBack`
- `GovernorThrottledSubsystem`

### Consequences

- Features can react to events without direct dependencies.
- Testing can inject event streams.
- The event bus must avoid becoming an unbounded logging system.
- Telemetry aggregation is necessary for performance.

## ADR-0007: Local SQLite Store and Personal Game Graph Foundation

**Status:** Accepted

### Context

The app is local-first and needs persistent state for discovered games, sessions, recommendations, applied actions, telemetry summaries, and future personal intelligence.

### Decision

Use SQLite for MVP. Model the data relationally while leaving a path toward a Personal Game Graph.

Initial entities:

- `devices`
- `hardware_snapshots`
- `games`
- `game_instances`
- `launch_profiles`
- `sessions`
- `telemetry_summaries`
- `process_observations`
- `recommendations`
- `optimization_actions`
- `rollback_points`
- `module_knowledge_items`
- `user_preferences`
- `ignore_rules`

Do not introduce a graph database for MVP. Represent graph-like relationships with typed tables and edges where needed.

### Consequences

- SQLite is simple, local, reliable, and easy to back up.
- Future graph reasoning can start from the same persisted data.
- Schema migrations must be added from day one.
- Cursor should not use ad hoc JSON files as the main product database.

## ADR-0008: Rules-First Recommendation Engine

**Status:** Accepted

### Context

The README repeatedly says deterministic systems should come before AI. The app must be fast and explainable.

### Decision

Build a rule engine before any AI runtime. Rules consume normalized inputs and emit recommendation candidates. The recommendation engine scores, deduplicates, prioritizes, and stores recommendations.

Rule inputs:

- Hardware profile
- Process observations
- Session telemetry summaries
- Launch profile
- Minecraft instance metadata
- Modpack analysis results
- User preferences
- Historical outcomes

Rule output:

- Recommendation candidate
- Evidence
- Confidence
- Risk
- Expected impact
- Optional action request

Initial rule packs:

- Windows background process rules
- Java/JVM rules
- Minecraft launcher/instance rules
- Minecraft performance mod rules
- Minecraft config rules
- App self-performance rules

### Consequences

- MVP can ship without AI.
- Recommendations are testable.
- The rule format should be data-driven where possible but can start as typed Rust rules for speed.
- Later local AI can explain or summarize rule outcomes rather than replace the rule engine.

## ADR-0009: Performance Governor

**Status:** Accepted

### Context

The app exists to improve game performance, so it must not become part of the problem. This is the central architectural constraint.

### Decision

Implement a Performance Governor that monitors GamePilot's own CPU, RAM, GPU where available, disk, and scheduling behavior. It can throttle or suspend nonessential subsystems.

Subsystem modes:

- `normal`
- `lite`
- `minimal`
- `paused`

Subsystems governed:

- Telemetry sampling
- Deep process scans
- Modpack analysis
- Knowledge indexing
- OCR, future
- AI, future
- Cloud sync, future
- UI animations if needed

Targets:

| Mode | CPU | RAM | GPU |
| --- | --- | --- | --- |
| Idle | <0.25% | <150 MB | ~0% |
| Monitoring | <1% | <250 MB | <1% |
| Burst | <3% | <500 MB | <2% |

### Consequences

- Governor hooks must be part of subsystem interfaces from the start.
- Performance budget tests are required early.
- Expensive work must be schedulable, cancelable, and resumable.
- A feature that cannot degrade gracefully should not run during gameplay.

## ADR-0010: Telemetry Strategy

**Status:** Proposed

### Context

GamePilot needs runtime insight without heavy overhead. FPS and frame time are valuable, but availability varies by game and integration.

### Decision

Use a layered telemetry model:

1. Always available: app process self-metrics, system CPU/RAM, target process CPU/RAM, process list snapshots.
2. Windows available where feasible: GPU, VRAM, disk I/O, network, temperature/power where APIs permit.
3. Game/module-specific: Java heap, Minecraft logs, Spark/profile exports, launcher logs, server/TPS signals.
4. FPS/frame time: use PresentMon/ETW-compatible strategy where viable; otherwise mark unavailable and still generate reports.

Persist summaries, not raw high-frequency samples, by default.

### Consequences

- MVP can still be useful if FPS capture is delayed.
- Telemetry code needs capability detection and clear "unavailable" states.
- Reports should explain missing signals rather than failing.
- PresentMon integration should be isolated behind an interface.

## ADR-0011: Minecraft MVP Module

**Status:** Accepted

### Context

Minecraft is the first target and exercises the full product loop: discovery, configuration, launch, telemetry, recommendations, and modpack analysis.

### Decision

Build a Minecraft module with these responsibilities:

- Discover launchers and instances.
- Parse instance metadata.
- Detect loader and Minecraft version.
- Detect Java/runtime configuration.
- Parse mods folder and manifests.
- Build dependency graph summary.
- Identify known performance mods.
- Emit Minecraft-specific rule inputs.
- Request safe JVM/config actions.
- Launch instance through supported launch method.
- Summarize session data with Minecraft-specific context.

The module must not directly close processes, edit files without core approval, or own rollback.

### Consequences

- The module is a strong test case for the capability model.
- Minecraft-specific behavior stays isolated.
- The first version of modpack intelligence should be useful but conservative.

## ADR-0012: Minecraft Mod Metadata and Rule Packs

**Status:** Proposed

### Context

The long-term vision includes a mod metadata database with dependencies, conflicts, memory/CPU/GPU impact, safe removal status, and replacements. Building a complete database is too large for MVP.

### Decision

Start with a curated local rule pack and metadata format. Include common performance mods, common launch/JVM rules, and high-confidence configuration rules. Keep the format versioned and updateable later.

Initial metadata fields:

- Mod id
- Display name
- Loader
- Side
- Dependencies
- Optional dependencies
- Known conflicts
- Performance category
- Client/server impact
- Safe removal classification
- Recommended alternatives
- Notes
- Source/reference optional

Initial classifications:

- `performance_recommended`
- `known_heavy`
- `dependency_library`
- `content_mod`
- `client_only`
- `server_only`
- `unknown`

### Consequences

- MVP does not need full community-scale mod intelligence.
- Unknown mods can be reported honestly.
- The format can later support cloud/community updates.
- Cursor should avoid scraping mod websites as an MVP dependency.

## ADR-0013: Safety and Rollback Model

**Status:** Accepted

### Context

Trust is central. The app will touch game configs and launch settings. Users must know what changed and be able to undo it.

### Decision

Every optimization action follows this lifecycle:

1. Preview
2. Confirm
3. Backup
4. Apply
5. Validate
6. Record outcome
7. Rollback if requested or validation fails

For file changes, store a rollback point with original content hash, backup path, action metadata, timestamp, and validation result.

For process recommendations, default to recommendation-only. If process closing is implemented, require explicit user action and do not close system-critical or protected processes.

### Consequences

- Optimization work takes longer to implement but is safer.
- UI must make action state visible.
- Automated tests should cover rollback success and failure paths.
- High-risk actions can be deferred without blocking the MVP.

## ADR-0014: Game State Providers and Compliance

**Status:** Accepted

### Context

The transcript considers reading game memory but correctly identifies anti-cheat, fragility, and compliance risk. The architecture needs a safer abstraction.

### Decision

Use Game State Providers. Providers normalize game state from permitted sources:

- Official APIs
- Plugin APIs
- Logs
- Config files
- Save files
- Local databases
- User exports
- OCR/computer vision
- Telemetry
- Read-only memory inspection only after explicit compliance review

The recommendation engine consumes normalized state. It does not know or care which provider produced it.

Memory inspection is not part of the MVP.

### Consequences

- Game modules can choose the safest source for each game.
- Anti-cheat-sensitive work is isolated and deferred.
- Future League support should prefer Riot APIs and Live Client Data where policy permits.
- Future OCR support can plug into the same provider model.

## ADR-0015: Knowledge Engine and Personal Intelligence

**Status:** Proposed for foundation, deferred for full feature set

### Context

The long-term differentiator is personalized intelligence. GamePilot should learn from sessions and recommendations to provide advice "based on you."

### Decision

Create a Knowledge Engine interface and local schema foundation in MVP, but defer advanced planners, heatmaps, and community intelligence.

MVP foundation:

- Store game/module knowledge items.
- Store session outcomes.
- Store recommendation outcomes.
- Store user preferences and ignore rules.
- Preserve relationships needed for future Personal Game Graph.

Future Personal Intelligence Engine:

- Build long-term player profile.
- Track strengths, weaknesses, habits, and trends.
- Support spatial analytics and heatmaps where game data allows.
- Compare opt-in anonymized cohorts.
- Personalize recommendations using player history.

### Consequences

- MVP data schema should not be throwaway.
- The UI can hint at trends without overpromising future AI.
- Advanced gameplay coaching stays out of the first build.

## ADR-0016: Optional AI Runtime

**Status:** Deferred

### Context

The user wants an optional lightweight model that can reason over OCR/logs/events in real time, but the app cannot be resource intensive.

### Decision

No AI runtime in MVP. Design interfaces so future AI can consume events and recommendation context.

When added, AI must be:

- Optional
- Local-first where practical
- Event-driven, not frame-driven
- Suspended by the Performance Governor
- Used for explanation, summarization, classification, and ambiguous reasoning
- Never required for core launch, optimization, or reporting

### Consequences

- MVP stays shippable and performant.
- Rule engine must be strong enough to stand alone.
- AI integration points should be explicit but inactive.

## ADR-0017: Cloud and Community Intelligence

**Status:** Deferred

### Context

Cloud sync, anonymous benchmarks, shared optimization profiles, and community intelligence are valuable future features, but they introduce privacy, security, moderation, and backend complexity.

### Decision

No required cloud dependency in MVP. Design local identifiers and data export paths so opt-in cloud can be added later.

Future cloud capabilities:

- Profile sync
- Settings sync
- Anonymous benchmark sharing
- Community optimization rule packs
- Hardware-class recommendations
- Mod metadata/rule updates

### Consequences

- MVP works offline.
- User trust is easier to establish.
- Backend work can happen after local product value is proven.

## ADR-0018: UX Architecture

**Status:** Accepted

### Context

The user wants a modern, high-fidelity, responsive native desktop app that is easy to navigate and read. The app is operational software, not a marketing site.

### Decision

Use an app-shell layout:

- Left navigation rail or compact sidebar.
- Dashboard as the first screen.
- Game detail pages for each discovered game.
- Persistent top status area for current scan/session state.
- Recommendation drawer or list with filters.
- Session report pages.
- Settings and diagnostics.

Design principles:

- Dense but readable.
- No hero/landing page as the app entry point.
- Dark mode first with light mode support.
- Clear status indicators and confidence labels.
- Action buttons use icons and concise labels.
- Avoid nested cards and decorative clutter.
- Every recommendation answers: what, why, expected gain, risk, rollback.

### Consequences

- UI work should be treated as a first-class product surface.
- The design system should be implemented early.
- The app should feel like a performance cockpit, not a generic SaaS dashboard.

## ADR-0019: Testing and Verification Strategy

**Status:** Accepted

### Context

GamePilot will interact with user files, processes, launchers, and system metrics. Bugs can break trust quickly.

### Decision

Testing layers:

- Rust unit tests for rules, parsers, schemas, action validation, and rollback.
- Integration tests with fixture Minecraft instances and mock launcher paths.
- UI component tests for recommendation rendering and action states.
- End-to-end tests for discovery, recommendation preview, apply, rollback, and report rendering.
- Performance budget checks for baseline app resource usage.
- Golden fixture tests for modpack analysis outputs.

Required fixture categories:

- Empty system/no Minecraft found.
- Prism instance with simple modpack.
- CurseForge-style manifest.
- Modrinth-style manifest.
- Broken/missing Java.
- High background process risk.
- Config edit rollback.

### Consequences

- Cursor should scaffold tests before deep feature work.
- System collectors need mockable interfaces.
- Performance budget verification should be automated as much as possible.

## ADR-0020: Initial Engineering Work Breakdown

**Status:** Proposed

### Context

The user wants docs that a Cursor agent can break into tasks. The implementation should proceed in slices that each produce testable software.

### Decision

Recommended build order:

1. Scaffold Tauri/Rust/React app with CI, formatting, tests, and packaging skeleton.
2. Add local SQLite migrations and typed data access.
3. Build desktop shell, dashboard, settings, diagnostics, and recommendation UI primitives.
4. Implement event bus and typed event contracts.
5. Implement hardware and process analyzers with mockable interfaces.
6. Implement recommendation schema, rule engine, and starter rules.
7. Implement optimization action preview/apply/rollback engine.
8. Implement Minecraft discovery.
9. Implement Java/JVM analysis and launch profile changes.
10. Implement Minecraft launch flow.
11. Implement modpack parser and starter metadata/rule pack.
12. Implement telemetry summaries and post-session reports.
13. Implement performance governor.
14. Harden installer, errors, permissions, accessibility, and performance tests.

### Consequences

- Each milestone can be assigned to a focused agent.
- Future AI/cloud/gameplay intelligence stays out of the critical path.
- The first useful product loop appears before the full platform vision is built.

## Cross-ADR Non-Negotiables

1. GamePilot must stay lightweight during gameplay.
2. The core must stay game-agnostic.
3. Minecraft MVP must be useful without AI or cloud.
4. Every applied optimization must be explainable and reversible.
5. The app must prefer official and compliant game state sources.
6. Recommendations must include evidence, confidence, impact, and risk.
7. Future features cannot justify an overbuilt MVP.

## Cursor Agent Handoff

Start by creating an implementation plan from `docs/PRD.md` and this ADR file. Do not implement all future platform features. Build the Minecraft MVP around the full product loop:

```text
Discover -> Diagnose -> Recommend -> Preview -> Apply -> Launch -> Monitor -> Report -> Learn
```

The first usable release should make one Minecraft session visibly better, safer, and more understandable while proving that the architecture can grow into the broader GamePilot platform.
