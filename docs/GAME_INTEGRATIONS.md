# GamePilot Game Integrations Research

**Status:** Research / Pre-implementation  
**Date:** 2026-07-12  
**Audience:** Implementation agents, architecture reviewers  
**Related:** `docs/PRD.md` (sections 8.1–8.5), `docs/ADR.md` (ADR-0003, ADR-0014)

This document catalogs the APIs, data sources, compliance constraints, and integration strategies for each game GamePilot intends to support beyond the Minecraft MVP.

---

## Table of Contents

1. [League of Legends](#1-league-of-legends)
2. [Escape from Tarkov](#2-escape-from-tarkov)
3. [RuneScape (OSRS and RS3)](#3-runescape-osrs-and-rs3)
4. [Path of Exile](#4-path-of-exile)
5. [General PC Games (Steam / Epic)](#5-general-pc-games-steam--epic)
6. [Game Module Architecture](#6-game-module-architecture)
7. [Priority Matrix](#7-priority-matrix)

---

## 1. League of Legends

### 1.1 Riot Games API

**Base URL:** `https://{region}.api.riotgames.com` (platform-routed) and `https://{cluster}.api.riotgames.com` (region-routed)

**Authentication:** API key passed via `X-Riot-Token` header or `api_key` query parameter. Development keys expire every 24 hours. Production keys require a registered, approved application on the Riot Developer Portal and do not expire.

**Rate Limits:**

| Key Type | Application Rate Limit | Method Rate Limits |
| --- | --- | --- |
| Development | 20 req/sec, 100 req/2 min | Per-endpoint (see below) |
| Personal | Same as development | Same |
| Production | Varies (starts ~500 req/10 sec) | Varies by endpoint |

Key method rate limits (production defaults):

| Endpoint | Rate Limit |
| --- | --- |
| `/lol/match/v5/matches/{matchId}` | 500 req/10 sec |
| `/lol/match/v5/matches/by-puuid/{puuid}/ids` | 1,000 req/10 sec |
| `/lol/summoner/v4/summoners/*` | 1,000 req/1 min (global) |
| `/lol/league/v4/entries/*` | 300 req/1 min (varies by region) |

Rate limit state is communicated via response headers `X-App-Rate-Limit`, `X-App-Rate-Limit-Count`, `X-Method-Rate-Limit`, and `X-Method-Rate-Limit-Count`. A `429` response includes a `Retry-After` header.

**Key Endpoints:**

| Capability | Endpoint | Data Returned |
| --- | --- | --- |
| Account lookup | `GET /riot/account/v1/accounts/by-riot-id/{gameName}/{tagLine}` | PUUID, gameName, tagLine |
| Summoner data | `GET /lol/summoner/v4/summoners/by-puuid/{puuid}` | summonerId, profileIconId, summonerLevel |
| Match history | `GET /lol/match/v5/matches/by-puuid/{puuid}/ids` | List of match IDs |
| Match detail | `GET /lol/match/v5/matches/{matchId}` | Full match data: participants, stats, items, runes, timeline |
| Champion mastery | `GET /lol/champion-mastery/v4/champion-masteries/by-puuid/{puuid}` | Per-champion mastery points, level, last play |
| Ranked stats | `GET /lol/league/v4/entries/by-summoner/{summonerId}` | Tier, rank, LP, wins, losses per queue |
| Active game | `GET /lol/spectator/v5/active-games/by-summoner/{puuid}` | Current game participants, champions, runes, summoner spells |
| Static data | Data Dragon CDN: `https://ddragon.leagueoflegends.com/cdn/{version}/data/en_US/` | Champions, items, runes, summoner spells (JSON + images) |

**Example Request:**

```
GET https://americas.api.riotgames.com/riot/account/v1/accounts/by-riot-id/Doublelift/NA1
X-Riot-Token: RGAPI-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
```

**Example Response (match detail, abbreviated):**

```json
{
  "metadata": {
    "matchId": "NA1_4567890123",
    "participants": ["puuid1", "puuid2"]
  },
  "info": {
    "gameDuration": 1847,
    "gameMode": "CLASSIC",
    "participants": [
      {
        "puuid": "puuid1",
        "championName": "Jinx",
        "kills": 8,
        "deaths": 3,
        "assists": 12,
        "totalDamageDealtToChampions": 28450,
        "item0": 3031,
        "win": true
      }
    ]
  }
}
```

**Free vs Requires Approval:**

- Development key: free, 24h expiry, low rate limits. Sufficient for local personal use.
- Personal key: free, requires registration, same rate limits as dev but persistent.
- Production key: requires full application with website, description, privacy policy, screenshots. ~20 business day review. Higher rate limits.
- RSO (Riot Sign On): required for accessing authenticated user data. Only available to approved production applications.

### 1.2 Live Client Data API

**Base URL:** `https://127.0.0.1:2999/liveclientdata/`

This API runs locally on the player's machine while a League game is active. It uses a self-signed certificate (requires SSL verification bypass or Riot root certificate installation). No API key required — localhost only.

**Endpoints:**

| Endpoint | Data Returned |
| --- | --- |
| `GET /allgamedata` | Complete game state (superset of all below) |
| `GET /activeplayer` | Active player stats: level, gold, abilities, runes, full stats |
| `GET /activeplayerabilities` | Active player ability levels and details |
| `GET /activeplayerrunes` | Active player rune page |
| `GET /playerlist` | All 10 players: champion, team, scores, items, summoner spells, runes |
| `GET /playerscores?riotId={name}` | KDA and creep score for a specific player |
| `GET /playeritems?riotId={name}` | Current items for a specific player |
| `GET /playersummonerspells?riotId={name}` | Summoner spells for a specific player |
| `GET /eventdata` | Game events: kills, dragons, barons, turrets, inhibitors |
| `GET /gamestats` | Game time, map, mode, queue type |

**Key Constraint:** Data is only available while a game is running on the local machine. The API endpoint disappears when the game ends. Polling interval should be 1–5 seconds for UI updates.

### 1.3 Community Data Sources

| Source | API Available? | Usability for GamePilot |
| --- | --- | --- |
| **op.gg** | No public API. Terms prohibit scraping for commercial use without attribution. Help center says crawling is "not prohibited" but may be restricted for commercial use or excessive requests. | Not recommended. Fragile, compliance-risky. |
| **u.gg** | No public API. Terms prohibit scraping. | Not usable. |
| **lolalytics** | No public API. | Not usable. |

**Recommendation:** Use the Riot API directly for all player-specific data. For aggregate meta statistics (champion win rates, item build popularity), Riot's Match API provides the raw data to compute these locally or via a GamePilot backend service. Community sites like Data Dragon and Community Dragon provide static game asset data freely.

### 1.4 Compliance

**Allowed:**

- Using Riot API data to display player stats, match history, champion mastery.
- Using the Live Client Data API for in-game companion features.
- Using Data Dragon and Community Dragon static assets.
- Displaying performance analytics and post-game analysis.
- Recommending builds, runes, and strategies based on API data.

**Prohibited:**

- Creating MMR/ELO calculators or alternative ranking systems.
- Identifying or analyzing deliberately hidden (anonymous) players.
- Granting competitive advantage not present in the game client.
- Acting as a data broker between the Riot API and third parties.
- Memory reading or interaction with game process (Vanguard enforced — no allowlist, no exceptions).
- Using Riot IP beyond approved static assets.
- Cryptocurrency or blockchain integration.

**Required:**

- Product must be registered on the Riot Developer Portal.
- Legal disclaimer must be visible: "GamePilot is not endorsed by Riot Games and does not reflect the views or opinions of Riot Games..."
- One production key per product.
- API key must never be embedded in distributed code.
- SSL/HTTPS required for all API access.

### 1.5 What GamePilot Could Offer

| Feature | Data Source | Feasibility |
| --- | --- | --- |
| Pre-game build recommendations | Match API (aggregate win rates) + Data Dragon | High — compute locally from match data |
| Matchup analysis | Match API historical data per champion pair | High — requires batch ingestion |
| Real-time game companion | Live Client Data API | High — localhost, no API key |
| Post-game analysis | Match API + match timeline | High — detailed data available |
| Performance trends over time | Match API history + local DB | High — core GamePilot pattern |
| Champion mastery insights | Champion Mastery API | High — direct API |
| Settings optimization | Game config files (league of legends cfg) | Medium — file-based, game-specific |

### 1.6 Recommended Implementation Approach

1. Start with a Personal API key for development.
2. Implement account linking via Riot Sign On (RSO) for production.
3. Use the Live Client Data API for the real-time companion — this is the highest-value, lowest-barrier feature.
4. Build a local match history cache to avoid redundant API calls and respect rate limits.
5. Compute aggregate statistics (win rates, build effectiveness) locally from cached match data rather than depending on third-party sites.
6. Apply for production API key once the LoL module is feature-complete.

**Priority: HIGH — large user base, rich API, strong alignment with GamePilot's session-based analysis model.**

---

## 2. Escape from Tarkov

### 2.1 tarkov.dev API (Community)

**Base URL:** `https://api.tarkov.dev/graphql`

**Authentication:** None. Free, open, no API key required.

**Rate Limits:** Not formally documented. The API runs on Cloudflare Workers and is designed for public consumption. Reasonable use expected.

**Protocol:** GraphQL

**Available Data:**

| Query Root | Data Returned |
| --- | --- |
| `items` | All in-game items: name, price (flea market 24h avg), base price, dimensions, weight, categories, wiki link, icon |
| `ammo` | Ammunition: damage, penetration power, armor damage %, caliber, speed, fragmentation chance, recoil/accuracy modifiers, tracer info |
| `maps` | Map data: name, description, raid duration, player count, extracts, spawns, loot containers, boss spawns |
| `tasks` | Quests: objectives, requirements, rewards, trader unlocks, map associations |
| `traders` | Trader info: loyalty levels, reset times, available barters, currencies |
| `hideoutStations` | Hideout modules: construction requirements, crafts, bonuses |
| `crafts` | Crafting recipes: inputs, outputs, duration, station |
| `barters` | Barter trades: inputs, outputs, trader |
| `bosses` | Boss data: health, equipment, drops, spawn locations |
| `fleaMarket` | Flea market metadata: min level, commission rates |
| `status` | Server status: current status per service, status messages |
| `armorMaterials` | Armor material properties and durability stats |
| `playerLevels` | XP requirements per player level |
| `skills` | Player skills and leveling data |
| `lootContainers` | Container spawn data |

**Example Query:**

```graphql
query {
  ammo(lang: en, limit: 5) {
    item { name shortName }
    caliber
    damage
    penetrationPower
    armorDamage
    fragmentationChance
  }
}
```

**Example Response:**

```json
{
  "data": {
    "ammo": [
      {
        "item": { "name": "7.62x51mm M80", "shortName": "M80" },
        "caliber": "Caliber762x51",
        "damage": 80,
        "penetrationPower": 41,
        "armorDamage": 52,
        "fragmentationChance": 0.17
      }
    ]
  }
}
```

### 2.2 Battlestate Games Official APIs

BSG does not provide a public API. BSG's lead developer Nikita has mentioned plans for an official API to enable third-party tools and mobile apps, but as of mid-2026, no public endpoints have been released. The game's internal HTTP/HTTPS protocol has been reverse-engineered by community projects, but using it violates BSG's License Agreement (section 4.2) and risks account termination.

### 2.3 Community Tools

| Tool | Description | API? | Usability |
| --- | --- | --- | --- |
| **tarkov.dev** | Comprehensive game database, flea market prices, quest data | GraphQL API (free) | Primary data source |
| **tarkov-market.com** | Flea market price tracking | Limited API, formerly paid | Secondary, less reliable |
| **eft-api.tech** | Player profiles and stats via reverse-engineered endpoints | REST API (requires token) | Compliance-risky — based on unofficial game endpoints |
| **SPTarkov** | Single-player modding framework | N/A | Not relevant for live game integration |

### 2.4 Compliance Considerations

**BattlEye Anti-Cheat:**

- EFT uses BattlEye, a kernel-level anti-cheat system.
- BattlEye scans for DLL injection, memory editing, unauthorized file modifications, and suspicious processes.
- There is no allowlist or exception process for third-party tools.
- GamePilot must not read EFT process memory, inject into the game process, or modify game files.

**BSG Terms of Service:**

- Section 4.2 prohibits "use of any third-party software that allows you to replace, override or modify any existing game client files or data in the memory."
- There are no officially approved mods.
- Using reverse-engineered internal APIs risks account termination.

**Safe Approach for GamePilot:**

- Use only tarkov.dev's community API for game knowledge data (items, ammo, maps, quests, hideout).
- Do not interact with the EFT game client, process, or files in any way that could trigger BattlEye.
- Performance optimization features (hardware tuning, background process management) are safe as they do not touch the game.
- Personal analytics would require user-provided data (manual input, screenshots, OCR) since no compliant automated state acquisition path exists.

### 2.5 What GamePilot Could Offer

| Feature | Data Source | Feasibility |
| --- | --- | --- |
| Ammo comparison charts | tarkov.dev API | High — static data, rich detail |
| Loadout cost calculator | tarkov.dev API (items + prices) | High — compute from item/price data |
| Hideout planning (upgrade path, profit) | tarkov.dev API (hideout + crafts) | High — static + price data |
| Quest routing / checklist | tarkov.dev API (tasks) | High — objective/map data available |
| Map reference with extracts / loot | tarkov.dev API (maps) | Medium — data available, UI effort |
| Performance optimization | System-level (GamePilot core) | High — game-agnostic |
| Personal analytics (K/D, survival, profit) | User-provided data / OCR (future) | Low — no automated acquisition path |
| Barter and craft profit analysis | tarkov.dev API (barters + crafts + prices) | High — straightforward computation |

### 2.6 Recommended Implementation Approach

1. Integrate tarkov.dev GraphQL API as the primary knowledge provider.
2. Cache game data locally — items/ammo/maps change only on game patches.
3. Build the module as a companion/planning tool, not an in-game overlay.
4. Defer personal analytics until a compliant state acquisition method exists (OCR on post-raid screens, manual entry, or a future official BSG API).
5. Performance optimization (system-level) is safe and valuable since EFT is notoriously hardware-demanding.

**Priority: MEDIUM — strong user demand, excellent community API for game knowledge, but limited personal analytics without compliant state acquisition.**

---

## 3. RuneScape (OSRS and RS3)

### 3.1 Jagex Official APIs

**Hiscores (OSRS):**

| Endpoint | Format | Data Returned |
| --- | --- | --- |
| `GET https://secure.runescape.com/m=hiscore_oldschool/index_lite.json?player={name}` | JSON | All skills (rank, level, XP) + activities (rank, score) |
| `GET https://secure.runescape.com/m=hiscore_oldschool/index_lite.ws?player={name}` | CSV | Same data, CSV format |

Game mode variants: replace `hiscore_oldschool` with `hiscore_oldschool_ironman`, `hiscore_oldschool_hardcore_ironman`, `hiscore_oldschool_ultimate`, `hiscore_oldschool_deadman`, `hiscore_oldschool_seasonal`.

**Hiscores (RS3):**

| Endpoint | Format | Data Returned |
| --- | --- | --- |
| `GET https://secure.runescape.com/m=hiscore/index_lite.json?player={name}` | JSON | All skills + activities |
| `GET https://secure.runescape.com/m=hiscore/ranking.json?table={skill}&category={cat}&size={n}&topRank={start}` | JSON | Leaderboard: up to 50 players per request |

**Grand Exchange (OSRS):**

| Endpoint | Data Returned |
| --- | --- |
| `GET https://secure.runescape.com/m=itemdb_oldschool/api/catalogue/items.json?category=1&alpha={letter}&page={n}` | Items by first letter: name, id, icon, price, price trend |

**Grand Exchange (RS3):**

| Endpoint | Data Returned |
| --- | --- |
| `GET https://secure.runescape.com/m=itemdb_rs/api/catalogue/items.json?category={cat}&alpha={letter}&page={n}` | Items by category and letter |
| `GET https://secure.runescape.com/m=itemdb_rs/api/graph/{itemId}.json` | 180-day price history for an item |

**Authentication:** None. All endpoints are unauthenticated, public, read-only.

**Rate Limits:** Not formally documented. Jagex does not provide CORS headers, so requests must come from a backend/desktop context (not browser). Reasonable use assumed; excessive requests may be IP-blocked.

**Known Limitations:**

- No CORS headers — browser-based clients cannot call these APIs directly. Desktop apps (GamePilot/Tauri) can call them without issue.
- Hiscores return data only for players who appear on the hiscores (ranked players).
- GE API is paginated and limited in filtering capability.
- No endpoint for player inventory, bank, or equipment — these require plugin-based or user-provided data.

### 3.2 RuneLite Plugin API (OSRS)

RuneLite is an officially approved third-party OSRS client (on the Jagex Approved Client List alongside HDOS). It provides an extensive Java API for plugin development.

**Plugin Capabilities:**

| Capability | Description |
| --- | --- |
| Game state access | Current player stats, inventory, bank (when open), equipment, position, animation state |
| Overlay rendering | Draw overlays on the game client |
| UI panels | Add custom side panels to the client |
| Event hooks | React to game events: level ups, item changes, NPC interactions, chat messages |
| Config system | Per-user persistent plugin configuration |
| HTTP client | Make outgoing HTTP requests from within the plugin |

**Plugin Hub:** Community plugins are reviewed by RuneLite developers and (since April 2026) an automated review bot. Plugins must comply with Jagex's third-party client guidelines.

**Integration Strategy for GamePilot:**

GamePilot cannot run inside RuneLite (RuneLite is a Java game client, GamePilot is a Tauri app). Two integration paths:

1. **RuneLite plugin that sends data to GamePilot** — A lightweight RuneLite plugin exports game state (bank contents, current stats, equipment, GP) via localhost HTTP or a local file. GamePilot reads this data as a Game State Provider per ADR-0014.
2. **RuneLite plugin HTTP API** — RuneLite plugins can expose local HTTP endpoints. GamePilot polls these for state.

Both approaches require the player to have RuneLite installed and the GamePilot companion plugin active. This is a voluntary opt-in integration.

### 3.3 Wiki API

**OSRS Wiki:** `https://oldschool.runescape.wiki/api.php`

The wiki is hosted by Weird Gloop (independent from Jagex) and provides a MediaWiki API with the Bucket extension for structured data queries.

**Bucket API (recommended):**

```
GET https://oldschool.runescape.wiki/api.php?action=bucket&format=json&query=bucket('infobox_item').select('item_id','examine').where('item_name','Raw lobster').run()
```

Returns structured game data (items, monsters, quests, NPCs) without page scraping. Bucket names and fields are lowercase with underscores.

**RS3 Wiki:** `https://runescape.wiki/api.php` — same Bucket API structure.

**Key Buckets:**

| Bucket | Fields |
| --- | --- |
| `infobox_item` | item_id, item_name, examine, weight, high_alch, low_alch, buy_limit, members |
| `infobox_monster` | name, combat_level, hitpoints, attack_style, max_hit, slayer_level, slayer_xp |
| `infobox_bonuses` | attack_bonuses, defence_bonuses, prayer, strength |

The wiki is the officially linked game knowledge source for OSRS. Jagex has endorsed its use.

### 3.4 Compliance

**RuneLite's Status:**

- RuneLite is on the official Jagex Approved Client List. Using it is not a bannable offense.
- Jagex confirms RuneLite fully complies with their third-party client guidelines.
- Only RuneLite and HDOS are approved non-Jagex clients. All others are a breach of T&Cs.
- Plugins must pass review to ensure they do not violate game integrity rules.

**Jagex's Stance on Third-Party Tools:**

- Read-only companion tools that display game information are generally acceptable.
- Tools must not automate gameplay (botting), provide unfair advantages (tick-perfect prayer flicking automation), or circumvent game mechanics.
- The Jagex Launcher can run RuneLite directly.
- Jagex is developing an official RS3 plugin API (announced, under NDA with community developers).

**Safe for GamePilot:**

- Hiscores API, GE API, Wiki API: all public, no compliance concerns.
- RuneLite companion plugin: acceptable if it only reads and exports data (no automation, no advantage-granting overlays beyond what RuneLite already permits).
- Bank/inventory analysis from exported data: safe.

### 3.5 What GamePilot Could Offer

| Feature | Data Source | Feasibility |
| --- | --- | --- |
| Skill planner (XP to level, optimal training) | Hiscores API + Wiki API | High — well-structured data |
| Bank value analysis | RuneLite plugin + GE prices | Medium — requires plugin integration |
| Quest routing / requirements checker | Wiki API (quest data) | High — comprehensive wiki data |
| Gear optimization / loadout comparison | Wiki API (item bonuses) | High — static data |
| Money-making guide ranking | Wiki API + GE API | Medium — requires price + method data |
| GE price alerts and flip tracking | GE API | Medium — polling required |
| Performance optimization | System-level (GamePilot core) | High — game-agnostic |
| Personal progression trends | Hiscores API (periodic snapshots) | High — snapshot and diff |
| Slayer task optimization | Wiki API (monster data) | High — static data |

### 3.6 Recommended Implementation Approach

1. Start with Hiscores API + Wiki Bucket API for a read-only companion.
2. Build skill planning, quest routing, and gear comparison as purely data-driven features.
3. Develop a RuneLite companion plugin for bank/inventory export as a second phase.
4. GE price tracking can run on a timed schedule respecting rate limits.
5. OSRS is the primary target; RS3 can share most of the same architecture with different API paths.

**Priority: MEDIUM-HIGH — large dedicated playerbase, excellent public APIs, strong fit for GamePilot's planning/analytics model. No compliance friction for companion tools.**

---

## 4. Path of Exile

### 4.1 GGG Official API

**Base URL:** `https://api.pathofexile.com`

**Authentication:** OAuth 2.1 with PKCE support.

| Grant Type | Use Case | Requirements |
| --- | --- | --- |
| Authorization Code | Accessing user account data (stashes, characters) | Registered OAuth client, user consent |
| Client Credentials | Accessing service-level data (leagues, ladders, public stashes) | Registered OAuth client |

**Registration:** Email `oauth@grindinggear.com` with account name (including 4-digit discriminator), app name, client type, grant types, required scopes, and redirect URI. GGG reviews at their discretion — responses can be slow, especially around league launches.

**Scopes:**

| Scope | Access |
| --- | --- |
| `account:profile` | Basic account profile |
| `account:characters` | Character list, inventories, passive tree |
| `account:stashes` | Stash tabs and items (PoE1 only) |
| `account:leagues` | Available leagues including private |
| `account:league_accounts` | Atlas passives |
| `account:item_filter` | Item filter management |
| `service:leagues` | Public league data |
| `service:leagues:ladder` | League ladders |
| `service:psapi` | Public stash tab stream |
| `service:cxapi` | Currency exchange data |

**Key Endpoints:**

| Endpoint | Scope | Description |
| --- | --- | --- |
| `GET /character` | `account:characters` | List account characters |
| `GET /character/{name}` | `account:characters` | Full character detail: items, passives, jewels |
| `GET /stash/{league}` | `account:stashes` | List all stash tabs in a league (PoE1) |
| `GET /stash/{league}/{stash_id}` | `account:stashes` | Specific stash tab with items |
| `GET /league` | `service:leagues` | List active leagues |
| `GET /league/{id}/ladder` | `service:leagues:ladder` | League ladder |
| `GET /public-stash-tabs` | `service:psapi` | Public stash tab stream (river) |

**Rate Limits:** Dynamic, communicated via response headers. Not published as static values.

| Header | Meaning |
| --- | --- |
| `X-Rate-Limit-Policy` | Policy name for this endpoint |
| `X-Rate-Limit-Rules` | Comma-separated list of active rules (e.g., `ip`, `account`, `client`) |
| `X-Rate-Limit-{rule}` | `max_hits:period_seconds:penalty_seconds` |
| `X-Rate-Limit-{rule}-State` | `current_hits:period_seconds:active_penalty_seconds` |
| `Retry-After` | Seconds to wait on 429 |

Example: `X-Rate-Limit-Client: 10:5:10` means 10 requests per 5 seconds, with a 10-second penalty if exceeded.

**Compliance:**

- GGG's Terms of Use (section 7i) prohibit reverse-engineering endpoints outside their documentation.
- Only documented API endpoints and data exports may be used.
- Frequent rate limit violations will result in access revocation.
- Registration requests may be denied; GGG prioritizes stability.

### 4.2 poe.ninja (Community)

**Base URL:** `https://poe.ninja/api/data/`

**Authentication:** None. Free, public, no key required.

**Rate Limits:** Not formally documented. Reasonable use expected.

**Economy Data:**

| Endpoint Pattern | Category Examples |
| --- | --- |
| `GET /currencyoverview?league={league}&type={type}` | Currency, Fragment |
| `GET /itemoverview?league={league}&type={type}` | UniqueWeapon, UniqueArmour, SkillGem, DivinationCard, Scarab, Fossil, Oil, Essence, BaseType, Map, UniqueMap, UniqueFlask, UniqueJewel, UniqueAccessory, Beast, Incubator, Resonator |
| `GET /ItemHistory?league={league}&type={type}&itemId={id}` | Price history for a specific item |

**Build Data:**

| Endpoint | Description |
| --- | --- |
| `GET /data/GetBuildOverview?overview={league}&type=exp` | Builds sorted by XP (ladder characters) |
| `GET /data/GetBuildOverview?overview={league}&type=depthsolo` | Builds sorted by Delve solo depth |

**Swagger documentation:** `https://poe.ninja/swagger/index.html`

poe.ninja is widely used by the community and is the de facto standard for PoE economy data. It aggregates from GGG's public stash tab stream.

### 4.3 What GamePilot Could Offer

| Feature | Data Source | Feasibility |
| --- | --- | --- |
| Stash organization and valuation | GGG API (stashes) + poe.ninja (prices) | High — rich data on both sides |
| Build optimization / comparison | GGG API (characters) + poe.ninja (builds) | High — character + meta data |
| Currency tracking and alerts | poe.ninja (economy) | High — polled data |
| Craft profit calculator | poe.ninja (item prices) | Medium — requires craft knowledge rules |
| Passive tree analysis | GGG API (character passives) | Medium — tree data available, analysis logic complex |
| Performance optimization | System-level + game config files | High — PoE is notoriously resource-heavy |
| League progress tracker | GGG API (characters) + league ladder | Medium — periodic polling |

### 4.4 Recommended Implementation Approach

1. Register an OAuth client with GGG early — lead times are long.
2. Use poe.ninja for economy data; it is more accessible and lower-friction than the public stash tab stream.
3. Character and stash data require user OAuth consent — implement standard OAuth flow.
4. Cache economy data aggressively (poe.ninja data updates periodically, not real-time).
5. Parse rate limit headers on every GGG API response and implement adaptive backoff.
6. PoE1 and PoE2 have diverging APIs — scope initial work to PoE1 unless PoE2 API matures.

**Priority: MEDIUM — dedicated playerbase, good APIs, but OAuth registration bottleneck and complex game systems increase implementation effort.**

---

## 5. General PC Games (Steam / Epic)

### 5.1 Steam Web API

**Base URL:** `https://api.steampowered.com`

**Authentication:** API key (free, generated at `https://steamcommunity.com/dev/apikey`). Passed as `key` query parameter.

**Rate Limits:** Officially 100,000 requests/day. Undocumented per-method limits exist; 429 responses on excessive use. The partner endpoint (`https://partner.steam-api.com`) has stricter authentication but higher availability.

**Key Endpoints:**

| Interface / Method | Endpoint | Description |
| --- | --- | --- |
| `IPlayerService/GetOwnedGames` | `GET /IPlayerService/GetOwnedGames/v1/?key={key}&steamid={id}&include_appinfo=1` | Games owned by player: appid, name, playtime_forever, playtime_2weeks, icon |
| `IPlayerService/GetRecentlyPlayedGames` | `GET /IPlayerService/GetRecentlyPlayedGames/v1/?key={key}&steamid={id}` | Recently played games with playtime |
| `ISteamUserStats/GetPlayerAchievements` | `GET /ISteamUserStats/GetPlayerAchievements/v1/?key={key}&steamid={id}&appid={appid}` | Achievement list with unlock status and timestamps |
| `ISteamUserStats/GetSchemaForGame` | `GET /ISteamUserStats/GetSchemaForGame/v2/?key={key}&appid={appid}` | Game schema: achievements, stats definitions |
| `ISteamUserStats/GetUserStatsForGame` | `GET /ISteamUserStats/GetUserStatsForGame/v2/?key={key}&steamid={id}&appid={appid}` | Player stats for a game |
| `ISteamUser/GetPlayerSummaries` | `GET /ISteamUser/GetPlayerSummaries/v2/?key={key}&steamids={ids}` | Player profile: name, avatar, status, last logoff |
| `ISteamUser/GetFriendList` | `GET /ISteamUser/GetFriendList/v1/?key={key}&steamid={id}` | Friend list (public profiles only) |
| `ISteamApps/GetAppList` | `GET /ISteamApps/GetAppList/v2/` | Complete list of all Steam apps (no key required) |
| `IStoreService/GetAppList` | `GET /IStoreService/GetAppList/v1/?key={key}` | Store app list with more metadata |

**Example Response (GetOwnedGames, abbreviated):**

```json
{
  "response": {
    "game_count": 247,
    "games": [
      {
        "appid": 730,
        "name": "Counter-Strike 2",
        "playtime_forever": 4832,
        "playtime_2weeks": 120,
        "img_icon_url": "abc123"
      }
    ]
  }
}
```

**Constraints:** Player profiles must be set to public for most endpoints to return data (unless using the player's own API key). Service methods use `input_json` parameter encoding.

### 5.2 PCGamingWiki

**Base URL:** `https://www.pcgamingwiki.com/w/api.php`

**Protocol:** MediaWiki API with Cargo extension for structured queries.

**Authentication:** None required. Custom User-Agent string required (generic UAs are blocked).

**Rate Limits:** 30 requests per minute. HTTP 429 on excess.

**Data Available:**

| Cargo Table | Key Fields |
| --- | --- |
| `Infobox_game` | Page name, genres, themes, developer, publisher, release dates, available platforms |
| `Video_settings` | Resolution, aspect ratio, windowed/fullscreen, vsync, FPS cap |
| `Audio` | Subtitles, configurable audio channels |
| `Input` | Key remapping, controller support, touchscreen |
| `Network` | Multiplayer type, netcode, server browser |
| `System_requirements` | Min/recommended CPU, GPU, RAM, storage by OS |

**Redirect API:** `https://pcgamingwiki.com/api/appid.php?appid={steamAppId}` — redirects to the wiki page for a given Steam app.

**Value for GamePilot:** PCGamingWiki is the best structured source for PC game settings metadata (video settings, input, system requirements, known issues, fixes). This data powers settings optimization recommendations.

### 5.3 IGDB

**Base URL:** `https://api.igdb.com/v4`

**Authentication:** Twitch OAuth Client ID + Bearer token (IGDB is owned by Twitch/Amazon). Free tier available.

**Rate Limits:** 4 requests per second.

**Protocol:** POST requests with field-selection body syntax.

**Example:**

```
POST https://api.igdb.com/v4/games
Headers: Client-ID: {id}, Authorization: Bearer {token}
Body: fields name, genres.name, platforms.name, rating; where id = 1942;
```

**Data:** Comprehensive game metadata — names, genres, platforms, release dates, ratings, screenshots, cover art, involved companies, game modes. Does not include settings optimization data.

### 5.4 RAWG

**Base URL:** `https://api.rawg.io/api`

**Authentication:** API key as `key` query parameter. Free tier: 20,000 requests/month.

**Data:** Similar to IGDB — game metadata, descriptions, genres, platforms, Metacritic scores, system requirements, screenshots, stores. Slightly more accessible than IGDB for basic game discovery.

### 5.5 ProtonDB

**Base URL:** `https://www.protondb.com/api/v1/reports/summaries/{appid}.json`

**Authentication:** None.

**Rate Limits:** Not documented. Static JSON endpoints.

**Data:**

```json
{
  "tier": "gold",
  "confidence": "strong",
  "score": 0.72,
  "total": 245,
  "trendingTier": "platinum",
  "bestReportedTier": "platinum"
}
```

Tiers: `native`, `platinum`, `gold`, `silver`, `bronze`, `borked`, `pending`.

Community reports (detailed) require fetching hash mappings from `https://www.protondb.com/data/counts.json`, then accessing `https://www.protondb.com/data/reports/{device}/app/{hash}.json`.

**Value for GamePilot:** Linux gaming compatibility ratings. Relevant if/when GamePilot supports Linux (currently Windows-first per ADR-0002).

### 5.6 What GamePilot Could Offer

| Feature | Data Source | Feasibility |
| --- | --- | --- |
| Game library with unified metadata | Steam API + IGDB/RAWG | High — straightforward aggregation |
| Playtime and achievement tracking | Steam API | High — direct endpoints |
| Per-game settings optimization | PCGamingWiki + local config detection | Medium-High — structured wiki data + file parsing |
| Performance profiling | System-level telemetry during any game | High — game-agnostic core capability |
| Launch optimization (background processes) | GamePilot core | High — already built for Minecraft |
| Linux compatibility info | ProtonDB | Low priority (Windows-first) |
| Friend activity / social | Steam API (friend list) | Low priority |

### 5.7 Recommended Implementation Approach

1. Implement Steam library discovery via `GetOwnedGames` — this gives GamePilot its game library for non-Minecraft games.
2. Use PCGamingWiki Cargo API for per-game settings metadata (video settings, known issues, fixes).
3. Use IGDB or RAWG for game cover art, descriptions, and metadata enrichment.
4. Build a generic "PC Game" module that handles launch, performance monitoring, and settings analysis using PCGamingWiki data as the rule source.
5. ProtonDB integration deferred until Linux support is prioritized.

**Priority: MEDIUM — broad value proposition, but shallow per-game intelligence compared to dedicated game modules. Best deployed after at least one deep game module (LoL or OSRS) proves the pattern.**

---

## 6. Game Module Architecture

Per ADR-0003, each game module declares capabilities it supports. The core platform defines trait interfaces; modules implement the subset relevant to their game and available data sources.

### 6.1 GameModule Trait

```rust
pub trait GameModule: Send + Sync {
    /// Unique identifier for this game module (e.g., "league_of_legends", "osrs")
    fn id(&self) -> &str;

    /// Human-readable game name
    fn display_name(&self) -> &str;

    /// Detect whether this game is installed on the system
    fn discover_game(&self) -> Vec<GameInstance>;

    /// Discover launchable instances (accounts, profiles, regions)
    fn discover_instances(&self) -> Vec<Instance>;

    /// Return current game state from available providers (API, files, plugins)
    fn provide_state(&self) -> GameState;

    /// Return game-specific rules for the recommendation engine
    fn provide_rules(&self) -> Vec<Rule>;

    /// Analyze current configuration and return findings
    fn analyze_configuration(&self) -> Analysis;

    /// Generate game-specific optimization recommendations
    fn recommend_optimizations(&self) -> Vec<Recommendation>;

    /// Launch the game through the appropriate method
    fn launch(&self, profile: &LaunchProfile) -> LaunchResult;

    /// Summarize a completed session with game-specific context
    fn summarize_session(&self, session: &Session) -> SessionSummary;
}
```

### 6.2 Game State Provider Model (ADR-0014)

Each module declares its state providers. The core does not care which provider produced the state — it consumes normalized `GameState`.

```rust
pub enum StateProviderKind {
    OfficialApi,       // Riot API, Jagex Hiscores, GGG API, Steam API
    PluginApi,         // RuneLite plugin, Live Client Data API
    LogFile,           // Game logs, crash logs
    ConfigFile,        // Game settings files
    SaveFile,          // Local save data
    LocalDatabase,     // Game's local DB (e.g., SQLite)
    UserExport,        // User-provided data export
    Ocr,               // Screen capture analysis (future)
    Telemetry,         // System-level performance telemetry
}
```

### 6.3 Module Capability Matrix

| Capability | Minecraft | League of Legends | Escape from Tarkov | RuneScape (OSRS) | Path of Exile | Generic PC |
| --- | --- | --- | --- | --- | --- | --- |
| `discover_game` | Launcher scan | Riot Client detection | BSG Launcher detection | RuneLite / Jagex Launcher detection | Standalone client detection | Steam library API |
| `discover_instances` | Modpack instances | Accounts/regions | Accounts | Characters / game modes | Leagues / characters | Per-game |
| `provide_state` | Config files, logs, mods | Riot API + Live Client Data | tarkov.dev (knowledge only) | Hiscores API + RuneLite plugin | GGG API (OAuth) | Steam API + config files |
| `provide_rules` | JVM, mods, configs | Builds, runes, matchups | Ammo, loadout, hideout | Skills, gear, quests | Passive tree, items | Settings via PCGamingWiki |
| `analyze_configuration` | Modpack health, JVM args | Rune/item build analysis | Loadout cost/effectiveness | Gear bonuses, skill efficiency | Build viability | Video/audio settings |
| `recommend_optimizations` | JVM, mods, config tweaks | Build/rune suggestions, pre-game | Ammo selection, craft profit | Training methods, gear upgrades | Item upgrades, passive pathing | Settings optimization |
| `launch` | Direct / launcher delegation | Riot Client | BSG Launcher | RuneLite / Jagex Launcher | Standalone / Steam | Steam / direct |
| `summarize_session` | FPS, memory, JVM stats | KDA, damage, gold, vision | N/A (no automated data) | XP gained, levels, GP | Currency, XP, maps completed | Playtime, achievements |

### 6.4 Data Flow

```
Game Module                          Core Platform
┌──────────────┐                    ┌──────────────────────┐
│              │  GameState         │                      │
│  State       ├───────────────────>│  Recommendation      │
│  Providers   │                    │  Engine              │
│              │  Rule[]            │    │                  │
│  Rule        ├───────────────────>│    ├─> Score          │
│  Provider    │                    │    ├─> Deduplicate    │
│              │  Analysis          │    └─> Prioritize     │
│  Analyzer    ├───────────────────>│         │             │
│              │                    │         v             │
│              │  Recommendation[]  │  Recommendation[]     │
│              │<───────────────────│         │             │
│              │                    │         v             │
│              │  ActionRequest[]   │  Optimization Engine  │
│              ├───────────────────>│    ├─> Preview        │
│              │                    │    ├─> Backup         │
│              │                    │    ├─> Apply          │
│              │                    │    ├─> Validate       │
│              │                    │    └─> Rollback       │
│              │                    │                      │
│  Launcher    │  LaunchResult      │  Session Manager     │
│              ├───────────────────>│    ├─> Telemetry      │
│              │                    │    └─> Reports        │
└──────────────┘                    └──────────────────────┘
```

### 6.5 Shared Infrastructure per Module

Each module gets from the core platform:

- **Event bus** subscription for lifecycle events
- **Local database** tables namespaced to the module
- **HTTP client** with rate limiting, retry, and caching
- **Performance governor** hooks for throttling during gameplay
- **Backup/rollback manager** for any file modifications
- **Credential storage** via OS keychain for API tokens

---

## 7. Priority Matrix

### Effort vs User Value

| Game | User Value | Implementation Effort | API Quality | Compliance Risk | Priority |
| --- | --- | --- | --- | --- | --- |
| **League of Legends** | Very High | Medium | Excellent (official, well-documented) | Low (clear policies, approved program) | **P1** |
| **RuneScape (OSRS)** | High | Medium | Good (public APIs + wiki) | Low (approved client ecosystem) | **P2** |
| **Path of Exile** | Medium-High | High | Good (official OAuth) | Low (clear policies) | **P3** |
| **Escape from Tarkov** | Medium | Low (knowledge) / High (analytics) | Good (community) / None (official) | Medium (no official API, strict anti-cheat) | **P3** |
| **Generic PC (Steam)** | Medium | Medium | Good (Steam API + PCGamingWiki) | Very Low | **P2** |

### Recommended Build Order

1. **Generic PC / Steam** — Steam library discovery gives GamePilot its universal game library. Low risk, broad value, enables the "universal platform" positioning. Build alongside or just after the Minecraft MVP.

2. **League of Legends** — Highest single-game user value. The Live Client Data API enables a compelling real-time companion with zero API key requirements. The Riot API provides deep post-game analysis. Clear compliance path.

3. **RuneScape (OSRS)** — Strong fit for GamePilot's planning/analytics model. Public APIs with no authentication friction. RuneLite companion plugin adds depth. The planning-tool nature (skill calculators, quest routing, bank analysis) is well-aligned with a desktop companion.

4. **Path of Exile / Escape from Tarkov** — Both are strong candidates but have higher barriers. PoE requires OAuth registration with uncertain timelines. Tarkov lacks official APIs for personal data. Both benefit from the game knowledge / companion pattern that earlier modules will have proven.

### Decision Criteria for New Game Modules

When evaluating whether to add a new game module:

1. **Does an official API exist?** Prefer games with documented, sanctioned APIs.
2. **Is compliant state acquisition possible?** Per ADR-0014, prefer official APIs, plugins, logs, config files, and user exports.
3. **Does the game benefit from the GamePilot loop?** (Discover → Diagnose → Recommend → Apply → Launch → Monitor → Report → Learn)
4. **Is the game resource-intensive on PC?** Performance optimization is GamePilot's core value proposition.
5. **Is the user base large enough to justify the module?** Niche games can be supported via the generic PC module.
