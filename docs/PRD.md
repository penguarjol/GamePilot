# GamePilot Product Requirements Document

**Status:** Proposed  
**Date:** 2026-07-08  
**Audience:** Cursor agent, implementation agents, product/architecture reviewers  
**Source:** Requirements extracted from `README.md` conversation transcript

## 1. Product Definition

GamePilot is a Windows-first universal gaming intelligence platform. It continuously learns about the player, their hardware, their software environment, and the games they play to deliver personalized performance optimization, progression guidance, analytics, and coaching before, during, and after gaming sessions.

The MVP must prove this concept with Minecraft. Minecraft is the first implementation because it has difficult performance constraints, multiple launcher ecosystems, JVM tuning, modpack complexity, server/client configuration variance, and a large need for explainable optimization.

GamePilot is not merely a launcher. Launching is one capability inside a broader local-first platform that discovers games, diagnoses bottlenecks, applies safe optimizations, records lightweight telemetry, and produces personalized recommendations.

## 2. Product Thesis

PC gamers should not need to search forums, compare conflicting guides, manually inspect task manager, tune obscure launch settings, or guess why a game runs poorly. GamePilot should give them a clear answer:

> Given this player's hardware, software environment, game configuration, historical sessions, and goals, what is the highest-value action to improve performance or gameplay outcomes, and why?

For Minecraft MVP, that answer is usually about Java, JVM flags, RAM allocation, background resource hogs, modpack configuration, performance mods, world/server settings, and post-session performance trends.

For the broader platform, the answer may be about League of Legends builds, RuneScape progression, Tarkov routes, inventory decisions, or other game-specific outcomes. The core product architecture should support those future use cases without polluting the core with game-specific logic.

## 3. Guiding Principles

1. **The optimizer must consume fewer resources than it saves.** If a feature harms gameplay performance, it must pause, degrade, or disable itself.
2. **Game-agnostic core.** The core platform must not contain Minecraft-specific logic. Minecraft ships as the first game module.
3. **Rules first, AI last.** Deterministic rules and heuristics should generate most recommendations. AI is optional and used for explanation, summarization, and ambiguous reasoning.
4. **Recommendation first.** GamePilot must not silently modify a user's computer. Every change needs explanation, expected impact, confidence, and rollback.
5. **Local first.** The app must work offline. Cloud sync, community intelligence, and anonymous benchmarking are opt-in future capabilities.
6. **Event driven.** Avoid continuous polling when event subscriptions, scheduled sampling, or launch/session lifecycle events are sufficient.
7. **Personal over generic.** Long-term recommendations should say "based on you," not "based on Reddit."
8. **Compliance first.** Game state acquisition must prefer official APIs, logs, files, plugins, and user exports. Memory inspection is allowed only where permitted, safe, and compliant with the game's terms and anti-cheat constraints.

## 4. Target Users

### 4.1 Primary MVP User

Minecraft players running large modpacks, especially players using Prism, CurseForge, Modrinth, ATLauncher, MultiMC, the official Minecraft Launcher, or custom instances.

Needs:

- One-click launch path that chooses sensible Java and JVM settings.
- Warnings about Chrome, Discord screen sharing, OBS, Steam downloads, OneDrive sync, antivirus scans, RGB utilities, and other resource hogs.
- Modpack and configuration analysis without needing to understand every mod.
- Clear recommendations that explain expected performance impact and risk.
- Lightweight session reports showing FPS, frame time, memory, CPU/GPU pressure, and improvement opportunities.

### 4.2 Secondary MVP User

Minecraft server owner or friend-group admin optimizing a heavy modpack such as All the Mods 10 for 6-12 concurrent players.

Needs:

- Client and server configuration guidance.
- Modpack performance reports.
- Recommendations around Java, memory, simulation distance, view distance, chunk loader limits, world pre-generation, and performance mods.
- A path toward curated "performance edition" modpacks with reversible changes.

### 4.3 Future Users

- League of Legends players who want build, rune, matchup, draft, and post-game recommendations using compliant data sources.
- RuneScape players who want bank, skill, quest, gear, and money-making planning.
- Escape from Tarkov players who want stash, loadout, route, hideout, market, and personal heatmap analytics.
- PC gamers who want a single app that diagnoses any game, not only Minecraft.

## 5. MVP Goals

The first production-quality release must make launching Minecraft through GamePilot clearly better than launching it directly.

MVP goals:

1. Discover Minecraft installations and launchers.
2. Detect hardware and relevant Windows environment details.
3. Detect resource-heavy background applications before and during play.
4. Recommend safe pre-launch optimizations with estimated impact.
5. Select or recommend Java runtime and JVM arguments.
6. Analyze Minecraft modpacks at a useful first-pass level.
7. Launch Minecraft through a GamePilot launch profile.
8. Collect lightweight session telemetry.
9. Generate post-session reports with actionable recommendations.
10. Persist local user, device, game, recommendation, and session history.
11. Stay inside strict resource budgets while a game is running.
12. Provide a modern, high-fidelity Windows desktop experience.

## 6. MVP Non-Goals

The MVP should not attempt to ship every future platform idea.

Non-goals:

- Full League, RuneScape, Tarkov, or other game intelligence modules.
- Cloud sync, accounts, social features, or community benchmark publishing.
- Continuous local LLM inference.
- Real-time overlay for all games.
- Automatic unsafe memory reading.
- Automatic mod removal without backup, diff preview, and explicit user approval.
- Automatic process killing without explicit user approval.
- Kernel drivers, anti-cheat bypasses, or privileged system manipulation.
- Full mod metadata coverage for every mod on day one.
- Guaranteed FPS uplift for every setup. The product must be honest about confidence and uncertainty.

## 7. Functional Requirements

### 7.1 Native Windows Desktop Shell

Requirements:

- Windows-first packaged desktop application.
- Fast startup and responsive navigation.
- Dashboard, Game Library, Minecraft page, Recommendations, Session Reports, Settings, and Diagnostics views.
- Dark mode first; light mode supported.
- High-DPI support.
- Keyboard accessible navigation.
- Clear empty, loading, error, and permission states.
- No landing page as the primary experience. The app opens directly into the operational dashboard.

Acceptance criteria:

- User can install and launch GamePilot on Windows.
- Dashboard appears in under 2 seconds on a typical gaming PC after first-run setup.
- User can navigate to Minecraft, diagnostics, and settings without reading onboarding text.
- UI never blocks game launch because a nonessential analysis is still running.

### 7.2 First-Run Setup

Requirements:

- Explain local-first data handling.
- Ask for only the permissions needed for scanning launchers, reading game configs, process telemetry, and optional app control.
- Let users opt out of telemetry categories that are not required.
- Show discovered hardware summary and Minecraft installations.

Acceptance criteria:

- User understands what GamePilot reads and what it changes.
- User can skip optional scans.
- No optimization is applied during setup without explicit confirmation.

### 7.3 Game and Launcher Discovery

MVP must discover Minecraft installations from:

- Prism Launcher
- CurseForge
- Modrinth App
- ATLauncher
- MultiMC
- Official Minecraft Launcher
- Custom folder selected by user

Future generic game discovery should support Steam, Epic, GOG, Battle.net, Riot Client, standalone games, and emulators through game modules and launch integrations.

Acceptance criteria:

- GamePilot lists discovered Minecraft instances with launcher, path, Minecraft version, loader, and last modified time.
- User can add a custom instance folder manually.
- Discovery failures are visible and recoverable.

### 7.4 Hardware and Environment Detection

Requirements:

- Detect CPU model, core/thread topology, clock behavior where available, and CPU load.
- Detect GPU model, VRAM, driver version where available, and utilization.
- Detect RAM total, available RAM, and memory pressure.
- Detect storage type, free disk space, and basic disk contention.
- Detect display refresh rate.
- Detect OS version and relevant Windows gaming settings where available.
- Avoid requiring administrator privileges for core MVP behavior.

Acceptance criteria:

- Hardware summary is available before launching Minecraft.
- Hardware data is normalized into the core data model.
- Missing data degrades gracefully with explanation.

### 7.5 Background Process Analyzer

Requirements:

- Detect resource-heavy background processes.
- Categorize by RAM, CPU, GPU, disk, and network contention.
- Identify common gaming-impact processes such as Chrome, Discord screen share, OBS, Steam downloads, OneDrive sync, antivirus scans, RGB utilities, launchers, updaters, and browsers.
- Recommend actions such as close, pause, disable screen share, pause download, or ignore.
- Estimate impact using confidence bands rather than false precision.
- Never close a process without user approval.

Acceptance criteria:

- Before launch, user sees a ranked list of resource risks.
- Each recommendation includes evidence, expected benefit, confidence, and action.
- User can ignore once or always ignore a specific process signature.

### 7.6 Minecraft Java and JVM Optimization

Requirements:

- Detect installed Java runtimes.
- Validate required Java version for a selected Minecraft/modpack version where rules exist.
- Recommend or select Java 21 for modern modpacks such as ATM10 when applicable.
- Recommend Xmx/Xms based on total RAM, modpack size, historical heap usage, and safety margin for OS/background processes.
- Recommend GC/JVM flags using curated rules.
- Support per-instance launch profile overrides.
- Back up previous launch settings before changes.

Initial RAM guidance:

| System RAM | Recommended Minecraft Xmx Range |
| --- | --- |
| 8 GB | 5-6 GB |
| 16 GB | 8-10 GB |
| 32 GB | 10-12 GB |
| 64 GB | 12-16 GB |

Acceptance criteria:

- User sees current and recommended JVM settings.
- User can apply changes with preview and rollback.
- GamePilot records what changed and why.

### 7.7 Minecraft Modpack Analysis

Requirements:

- Parse instance metadata, manifests, config folders, mods folder, resource packs, shader packs, and loader type.
- Identify Forge, NeoForge, Fabric, Quilt, Vanilla where possible.
- Extract mod ids, names, versions, side, dependencies, and optional dependencies where available.
- Build a dependency graph.
- Detect known performance mods and missing recommended performance mods.
- Detect obvious duplicate functionality and known problematic combinations where rules exist.
- Detect shader/resource-pack implications for performance.
- Score modpack health, memory risk, client rendering risk, server/tick risk, startup risk, and dependency risk.

MVP recommendation examples:

- "ModernFix is missing for this NeoForge modpack. Expected memory/startup improvement: medium. Confidence: high."
- "Your pack has three backpack mods. This may be duplicate functionality. Review before removing. Confidence: medium."
- "Simulation distance is 10 on a heavy modpack. Recommend 6-8 for smoother play. Confidence: high."

Acceptance criteria:

- User can run an analysis on an installed instance.
- Report contains dependency graph summary, detected risks, and prioritized recommendations.
- Recommendations are useful even when full curated metadata is unavailable.

### 7.8 Modpack Optimization Actions

MVP must be conservative.

Allowed MVP actions:

- Edit launch/JVM settings with backup.
- Recommend missing performance mods.
- Open download/source links where available.
- Suggest config changes with diff preview.
- Apply low-risk config changes with backup and rollback.

Deferred actions:

- Automatic mod removal.
- Automatic replacement of major content mods.
- Publishing performance-edition modpacks.
- Server-side pack redistribution workflows.

Acceptance criteria:

- Every applied action has preview, backup, apply, validate, and rollback states.
- Risk level is displayed before apply.
- User can restore previous state from the app.

### 7.9 Launch Profiles

Requirements:

- Create a GamePilot launch profile per discovered game/instance.
- Store pre-launch optimization actions and launch settings.
- Apply approved optimizations before launching.
- Launch via the correct underlying launcher or direct executable where supported.
- Detect game exit and restore temporary settings.

Acceptance criteria:

- User can launch a Minecraft instance from GamePilot.
- GamePilot shows pre-launch recommendations before launch.
- Session tracking begins when launch succeeds.
- GamePilot handles failed launch with logs and recovery hints.

### 7.10 Runtime Telemetry

Requirements:

- Collect lightweight metrics while a session is active.
- Target metrics: FPS/frame time where available, CPU, GPU, RAM, VRAM, disk, network, temperatures where available, Java heap where available, and process interference.
- Use adaptive sampling and event-driven triggers.
- Reduce telemetry frequency under resource pressure.
- Avoid continuous expensive scans.

Acceptance criteria:

- Telemetry stays within the performance budget.
- User can see live health without opening Task Manager.
- Missing FPS data does not break session reporting.

### 7.11 Session Reports

Requirements:

- Generate a report after each launched session.
- Include average FPS, 1% lows where available, frame-time stability, CPU/GPU/memory pressure, JVM heap observations, resource-hog interference, recommendations applied, and observed outcome.
- Compare against previous sessions for the same instance.
- Explain recommendations in plain language.

Acceptance criteria:

- User receives a post-session summary.
- Recommendations are prioritized by expected impact and confidence.
- Report links evidence to the recommendation.

### 7.12 Recommendation Model

Every recommendation must include:

- Unique id
- Title
- Description
- Target game/instance
- Category
- Severity
- Confidence
- Expected impact
- Evidence
- Proposed action
- Risk level
- Rollback strategy
- Status
- Created timestamp

Recommendation categories:

- Performance
- Stability
- Configuration
- Background process
- Java/JVM
- Modpack
- World/server
- Hardware/driver
- Gameplay intelligence, future

Acceptance criteria:

- UI can render all recommendations consistently.
- Recommendations can be accepted, ignored once, ignored always, or deferred.
- Historical recommendation outcomes feed future confidence.

### 7.13 Local Data and Personal Profile

Requirements:

- Store local profiles for users, devices, games, instances, sessions, recommendations, applied actions, and outcomes.
- Build toward a Personal Game Graph: player, hardware, games, sessions, events, knowledge, recommendations, actions, and outcomes.
- MVP should implement enough schema to avoid later rewrites.
- User can delete local data.

Acceptance criteria:

- Session history persists across app restarts.
- Recommendations can reference historical context.
- Data deletion removes local profile and session data.

## 8. Future Platform Requirements

These are not MVP, but the MVP architecture must not block them.

### 8.1 Universal Game Intelligence Framework

Each game module should implement shared capabilities where applicable:

| Capability | Minecraft | League | Tarkov | RuneScape |
| --- | --- | --- | --- | --- |
| Launch | Yes | Yes | Yes | Yes |
| Performance optimization | Yes | Yes | Yes | Yes |
| Hardware tuning | Yes | Yes | Yes | Yes |
| State discovery | Yes | Yes | Yes | Yes |
| Knowledge provider | Yes | Yes | Yes | Yes |
| Progression planner | Yes | Yes | Yes | Yes |
| Analytics | Yes | Yes | Yes | Yes |
| Session reports | Yes | Yes | Yes | Yes |
| Personal insights | Yes | Yes | Yes | Yes |

### 8.2 Game State Providers

Future modules may acquire state through:

- Official APIs
- Plugin APIs
- Logs
- Config files
- Save files
- Telemetry exports
- Local databases
- User-provided exports
- OCR/computer vision
- Read-only memory inspection only when permitted and compliant

The recommendation engine must consume normalized game state and should not care where the state originated.

### 8.3 Knowledge Engine

Future knowledge providers may include:

- Official docs and APIs
- Community wikis respecting licenses and usage policies
- Patch notes
- Public datasets
- Curated rule packs
- Community-submitted profiles after review

Data should normalize into structured relationships, not raw pages.

### 8.4 Personal Intelligence Engine

Future responsibilities:

- Build long-term player profile.
- Track habits, strengths, weaknesses, and trends.
- Measure recommendation effectiveness.
- Feed personalized context into recommendations.
- Support personal heatmaps and spatial analytics where data exists.

Examples:

- Minecraft: "You usually quit worlds after AE2. These mods extend endgame automation."
- League: "You win 11% more often on scaling champions."
- Tarkov: "You survive more often with suppressed weapons and lose money on Lighthouse mid-raid."
- RuneScape: "You abandon travel-heavy Slayer tasks; this route minimizes downtime."

### 8.5 Optional AI

Future AI should:

- Be local-first where practical.
- Be optional.
- Run only on significant events or explicit user request.
- Explain, summarize, personalize, classify, and interpret logs/OCR.
- Never be required for core recommendations.
- Never continuously analyze frames or logs in a tight loop.

## 9. Performance Requirements

Targets:

| Mode | CPU | RAM | GPU | Disk | Network |
| --- | --- | --- | --- | --- | --- |
| Idle | <0.25% | <150 MB | ~0% | near zero | none unless enabled |
| Monitoring | <1% | <250 MB | <1% | minimal | optional |
| Burst | <3% | <500 MB | <2% | short-lived | user-controlled |

Performance behavior:

- Expensive work runs before game launch, after game exit, while PC is idle, or by explicit user request.
- During gameplay, nonessential work pauses under load.
- AI, OCR, metadata indexing, cloud sync, and deep scans are all suspendable.
- GamePilot monitors its own resource use.

## 10. Safety, Trust, and Privacy Requirements

Safety:

- No silent system modifications.
- No automatic process closing without approval.
- No irreversible game or config edits.
- Every applied action is logged.
- Backups are mandatory before file edits.
- Rollback is available from UI.

Trust:

- Each recommendation shows evidence.
- Confidence is explicit.
- Risk is explicit.
- Uncertainty is communicated honestly.

Privacy:

- Local-first by default.
- No cloud account required for MVP.
- Anonymous telemetry and benchmarking are opt-in.
- User can delete local data.
- Sensitive data should use OS-level secure storage where credentials or tokens are introduced.

Compliance:

- Prefer official integrations.
- Avoid anti-cheat-sensitive techniques.
- Memory inspection is not an MVP feature and must be governed by a future compliance review.

## 11. UX Requirements

GamePilot should feel like premium operational desktop software for gamers: fast, dense, readable, modern, and calm.

Required screens:

- Dashboard
- Game Library
- Minecraft Detail Page
- Recommendations
- Session Reports
- Diagnostics
- Settings

Dashboard must show:

- System health
- Discovered games
- Current hardware summary
- Running game/session
- Background process risks
- Top recommendations
- Recent sessions

Minecraft page must show:

- Launch button
- Instance list
- Java/JVM settings
- Modpack analysis
- Dependency graph summary
- Performance recommendations
- World/server analysis entries marked as future scope until implemented
- Applied changes and rollback

Session report must show:

- Performance summary
- Resource bottlenecks
- Recommendations applied
- Improvement or regression trend
- Next best actions

Interaction requirements:

- Users should understand "what is happening," "what should I do," "why," and "what improvement should I expect."
- Important actions should be one or two interactions away.
- Destructive or high-risk actions require confirmation.
- The app should not use in-app marketing copy as a substitute for usable controls.

## 12. Success Metrics

MVP product success:

- User can install app, discover Minecraft, and launch a known instance.
- User receives at least one accurate and actionable pre-launch recommendation on a realistic gaming PC.
- User receives a useful post-session report.
- User can apply and roll back a safe JVM/config optimization.
- App remains within monitoring performance budget during gameplay.
- App recovers gracefully when discovery, telemetry, or launch fails.

Potential quantitative metrics:

- Minecraft discovery success rate.
- Launch success rate.
- Recommendation acceptance rate.
- Rollback success rate.
- Average app CPU/RAM during gameplay.
- Number of useful recommendations per session.
- Session report generation success rate.
- User-reported usefulness of recommendations.

## 13. Suggested Milestones for Cursor Agent

### Milestone 0: Repository Foundation

- Choose implementation stack from ADR.
- Scaffold desktop app.
- Add lint, test, formatting, CI, and packaging skeleton.
- Add local database migration flow.
- Add performance budget test harness.

### Milestone 1: Desktop Shell and Local Data

- Build dashboard, navigation, settings, diagnostics, and reports shell.
- Implement local SQLite schema for devices, games, instances, sessions, recommendations, actions, and telemetry summaries.
- Add design system tokens and reusable UI primitives.

### Milestone 2: Hardware and Process Diagnostics

- Implement hardware detection.
- Implement process analyzer.
- Render resource-hog recommendations.
- Add ignore/always-ignore behavior.

### Milestone 3: Minecraft Discovery and Launch

- Discover major Minecraft launchers and manual instances.
- Parse core instance metadata.
- Implement launch profile.
- Launch selected instance and record session lifecycle.

### Milestone 4: Java/JVM Optimization

- Detect Java runtimes.
- Recommend Java/JVM settings.
- Apply settings with preview, backup, and rollback.
- Validate settings persisted correctly.

### Milestone 5: Modpack Analysis

- Parse mods and manifests.
- Build dependency graph summary.
- Implement curated starter rule pack for common performance mods and config risks.
- Render analysis report.

### Milestone 6: Runtime Telemetry and Session Reports

- Collect adaptive metrics.
- Enforce performance governor.
- Generate post-session reports.
- Compare against previous sessions.

### Milestone 7: Polish, Packaging, and Guardrails

- Harden error handling.
- Add installer.
- Add self-resource monitoring.
- Add accessibility pass.
- Add performance regression checks.

## 14. Open Product Decisions

The Cursor agent should resolve or propose these before implementation:

1. Exact Windows desktop stack.
2. Whether MVP uses direct game launch, launcher delegation, or both.
3. Initial telemetry source for FPS/frame time.
4. Scope of Java runtime management: detect only, bundled runtime, or download/install helper.
5. Initial mod metadata source and rule-pack format.
6. Whether first release includes a tray app/background daemon.
7. Code signing and installer strategy.
8. Exact data retention defaults.

## 15. Cursor Agent Instructions

The Cursor agent should treat this PRD and `docs/ADR.md` as the source of truth. First, break the work into implementation tasks by milestone. Do not start with future game intelligence, AI, OCR, cloud, or memory inspection. Build the smallest production-quality Minecraft MVP that proves GamePilot can safely diagnose, optimize, launch, monitor, and report while staying lightweight.

When deciding between feature scope and performance discipline, choose performance discipline. GamePilot loses its reason to exist if it competes with the game for resources.
