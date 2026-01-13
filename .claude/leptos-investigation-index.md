# Leptos Migration Investigation Index

## Constitutional Constraints (Active)

All investigations filtered through:

1. **Simple Made Easy**: Does this reduce entanglement? Is it simple (untangled) or merely easy (familiar)?
2. **Correct By Construction**: Does this leverage compile-time guarantees? Does type machinery earn its place?

## Hard Constraints

1. **NO JavaScript/TypeScript tooling** - Absolutely none. No node, deno, bun, npm, yarn, pnpm.
2. **Binaries only** - Use standalone binaries (e.g., tailwindcss standalone CLI) instead of JS-based tools.
3. **Pure Rust toolchain** - cargo-leptos + Rust binaries only.

---

## Investigation Priority Levels

| Level | Meaning | Action |
|-------|---------|--------|
| **P0** | Blocking architectural decisions | Clone + deep exploration |
| **P1** | Significant implementation guidance | Clone or thorough read |
| **P2** | Useful patterns, optional | Read documentation |
| **P3** | Reference material | Skim as needed |

---

## Category 1: Core Framework Documentation

### 1.1 Leptos MCP Documentation Sections

**Access**: `mcp__plugin_leptos-mcp_leptos__get-documentation` with section name

| Section | Priority | Why It Matters for Filmorator |
|---------|----------|-------------------------------|
| `signals` | P0 | Reactive ranking selector, undo functionality |
| `server-functions` | P0 | Type-safe S3/DB access, matchup fetching |
| `routing` | P0 | Campaign-aware routes `/{campaign_id}/compare` |
| `resources` | P0 | Async data loading for matchups, images |
| `components` | P1 | Photo card, ranking selector, lightbox |
| `views` | P1 | Dynamic classes for gold/silver/bronze borders |
| `actions` | P1 | Comparison submission POST |
| `forms` | P2 | Future: campaign creation UI |
| `suspense` | P2 | Loading states during matchup fetch |
| `error-handling` | P2 | ServerFnError patterns |
| `getting-started` | P3 | Basic setup reference |

### 1.2 Leptos Book

**URL**: https://leptos-rs.github.io/leptos/
**Source**: https://github.com/leptos-rs/leptos/tree/main/docs/book
**Priority**: P1
**Action**: Fetch key chapters via web, reference for patterns

---

## Category 2: Starter Templates (Clone + Explore)

### 2.1 start-axum (P0 - CRITICAL)

**Repo**: https://github.com/leptos-rs/start-axum
**Priority**: P0
**Action**: Clone to `~/pp/_leptos-exploration/start-axum`

**Investigation Questions**:
- [ ] Project structure: where does server vs client code live?
- [ ] How are server functions wired to Axum routes?
- [ ] cargo-leptos configuration (Cargo.toml, Leptos.toml)
- [ ] Asset handling (CSS, images)
- [ ] Build output structure
- [ ] How does hydration work?

### 2.2 start-axum-workspace (P0 - CRITICAL)

**Repo**: https://github.com/leptos-rs/start-axum-workspace
**Priority**: P0
**Action**: Clone to `~/pp/_leptos-exploration/start-axum-workspace`

**Investigation Questions**:
- [ ] Workspace structure: separate crates for what?
- [ ] Shared types between server/client
- [ ] How does this compare to single-crate approach?
- [ ] Which is better for filmorator's CLI+webapp split?

### 2.3 start-trunk (P2)

**Repo**: https://github.com/leptos-rs/start-trunk
**Priority**: P2
**Action**: Read README only

**Why lower priority**: CSR-only, we need SSR for SEO and initial load performance.

---

## Category 3: Example Applications (Clone + Study)

### 3.1 nicoburniske.com (P1 - PHOTO GALLERY - Source Not Public)

**Live**: https://nicoburniske.com
**Priority**: P1 (downgraded - no source available)
**Status**: Source code NOT PUBLIC

**Alternative**: Study the author's libraries instead:
- `leptos-image` (CLONED) - image optimization, LQIP
- `leptos-query` (CLONED) - async state management
- `tailwind-fuse` - class management

**Investigation via libraries**:
- [ ] leptos-image: Progressive loading patterns
- [ ] leptos-query: Data fetching patterns
- [ ] Infer gallery patterns from library usage

### 3.2 rustytube.rs (P1 - Media App)

**Repo**: https://github.com/nicoburniske/rustytube (search for actual repo)
**Live**: https://rustytube.rs
**Priority**: P1
**Action**: Clone for media handling patterns

**Investigation Questions**:
- [ ] How does it handle media assets?
- [ ] Desktop/web code sharing patterns
- [ ] Performance optimizations

### 3.3 leptos.dev Official Site (P1)

**Repo**: Part of leptos monorepo or separate?
**Live**: https://leptos.dev
**Priority**: P1
**Action**: Find source, study patterns

**Investigation Questions**:
- [ ] Production Leptos patterns
- [ ] SSR configuration
- [ ] Styling approach

### 3.4 simpleicons.org (P2)

**Live**: https://simpleicons.org
**Priority**: P2
**Action**: Study if source available

**Why relevant**: Large collection display, search/filter patterns (future ranking page)

---

## Category 4: Component Libraries (Evaluate)

### 4.1 Thaw UI (P1)

**Repo**: https://github.com/thaw-ui/thaw
**Docs**: Check for documentation site
**Priority**: P1
**Action**: Clone to `~/pp/_leptos-exploration/thaw-ui`

**Evaluation Criteria**:
- [ ] Component quality and completeness
- [ ] Styling approach (CSS-in-Rust? Tailwind? Custom?)
- [ ] Bundle size impact
- [ ] Accessibility
- [ ] **Simple Made Easy check**: Does it add or reduce entanglement?

### 4.2 Rust shadcn/ui - SKIP

**Status**: NOT USABLE - hyper-early pre-alpha, nothing works with Leptos yet

### 4.3 leptix (P2)

**Repo**: https://github.com/leptix/leptix
**Priority**: P2
**Action**: Read docs

**Why relevant**: Accessible, unstyled components (Radix-like)

---

## Category 5: Utility Libraries (Evaluate)

### 5.1 leptos-use (P0 - CRITICAL) - CLONED

**URL**: https://leptos-use.rs/
**Repo**: https://github.com/Synphonyte/leptos-use
**Local**: `~/pp/_leptos-exploration/leptos-use/`
**Priority**: P0
**Status**: CLONED

**Investigation Questions**:
- [ ] What reactive primitives does it provide?
- [ ] Intersection observer (for lazy image loading)?
- [ ] Local storage (for session persistence)?
- [ ] Debounce/throttle utilities?
- [ ] Does it follow Simple Made Easy? (values over state)

### 5.1b leptos-query (P1) - CLONED

**Repo**: https://github.com/gaucho-labs/leptos-query
**Local**: `~/pp/_leptos-exploration/leptos-query/`
**Priority**: P1
**Status**: CLONED

**Investigation Questions**:
- [ ] Async state management patterns
- [ ] Caching strategies
- [ ] SSR compatibility
- [ ] Query invalidation patterns

### 5.2 leptos_image (P1) - CLONED

**Repo**: https://github.com/gaucho-labs/leptos-image
**Local**: `~/pp/_leptos-exploration/leptos-image/`
**Priority**: P1
**Status**: CLONED

**Investigation Questions**:
- [ ] WebP conversion approach
- [ ] Progressive loading implementation
- [ ] LQIP (Low Quality Image Placeholders)
- [ ] Server-side vs client-side optimization
- [ ] Integration with S3/external storage

### 5.3 leptos-hotkeys (P2)

**Repo**: https://github.com/gaucho-labs/leptos-hotkeys
**Priority**: P2
**Action**: Read docs

**Why relevant**: VISION.md mentions keyboard shortcuts (1, 2, 3) for ranking

### 5.4 leptos_darkmode (P3)

**Repo**: https://gitlab.com/kerkmann/leptos_darkmode
**Priority**: P3
**Action**: Reference only

---

## Category 6: Styling Solutions (Evaluate)

### 6.1 Tailwind Integration - SOLVED (No JS Required!)

**Status**: FULLY SUPPORTED via standalone binary

**How it works** (from cargo-leptos source analysis):
1. cargo-leptos auto-downloads Tailwind standalone binary from GitHub releases
2. Binary cached in `~/Library/Caches/cargo-leptos/` (macOS)
3. **NO npm/node/bun required**

**Configuration** (in Cargo.toml):
```toml
[package.metadata.leptos]
tailwind-input-file = "style/tailwind.css"
# tailwind-config-file is OPTIONAL in v4
```

**Input file** (style/tailwind.css - Tailwind v4 syntax):
```css
@import "tailwindcss";
```

**Tailwind v4 note**: JS config files are no longer required. If you still need one, see: https://tailwindcss.com/docs/upgrade-guide#using-a-javascript-config-file

**Reference**: `/Users/ryzhakar/pp/_leptos-exploration/cargo-leptos/examples/project/`

### 6.2 Tailwind Fuse (P2)

**Repo**: https://github.com/gaucho-labs/tailwind-fuse
**Priority**: P2
**Action**: Read docs

**Why relevant**: Class management for conditional styling (gold/silver/bronze borders)

### 6.3 Stylance (P3) - SKIP

**Status**: Lower priority - Tailwind v4 with standalone binary is the clear winner

---

## Category 7: Server Integration Patterns

### 7.1 Custom State Injection - SOLVED

**Source**: `leptos/integrations/axum/src/lib.rs`
**Status**: PATTERN IDENTIFIED

**Pattern for filmorator's dual S3 clients + DB pool:**

```rust
use axum::extract::FromRef;
use leptos::prelude::*;

#[derive(Clone, FromRef)]
struct AppState {
    leptos_options: LeptosOptions,
    internal_s3: S3Client,    // minio:9000
    presign_s3: S3Client,     // localhost:9000
    db_pool: PgPool,
}

// In main.rs:
let app = Router::new()
    .leptos_routes_with_context(
        &app_state,
        routes,
        {
            let app_state = app_state.clone();
            move || {
                provide_context(app_state.clone());
            }
        },
        || shell(leptos_options.clone()),
    )
    .with_state(app_state);
```

**In server functions:**
```rust
#[server]
async fn get_matchup(campaign_id: String) -> Result<Matchup, ServerFnError> {
    let state = expect_context::<AppState>();
    let manifest = state.internal_s3.get_object(...).await?;
    // ...
}
```

**Key functions:**
- `leptos_routes_with_context` - provides additional context to routes
- `provide_context()` - makes state available in reactive scope
- `expect_context::<T>()` - retrieves state in server functions

**Reference**: `leptos/integrations/axum/src/lib.rs:1699-1707`

### 7.2 Server Functions Documentation

**Access via MCP**: `mcp__plugin_leptos-mcp_leptos__get-documentation` with section `server-functions`

**Key patterns to investigate:**
- [ ] Error handling with `ServerFnError`
- [ ] Streaming responses
- [ ] Custom extractors

---

## Category 8: Build & Tooling

### 8.1 cargo-leptos (P0 - CRITICAL)

**Repo**: https://github.com/leptos-rs/cargo-leptos
**Priority**: P0
**Action**: Clone to `~/pp/_leptos-exploration/cargo-leptos`

**Investigation Questions**:
- [ ] Leptos.toml configuration options
- [ ] Watch mode behavior
- [ ] Production build optimization
- [ ] Asset hashing/caching
- [ ] Integration with existing Axum app

### 8.2 leptosfmt (P2)

**Repo**: https://github.com/bram209/leptosfmt
**Priority**: P2
**Action**: Read docs, install

**Why relevant**: Code formatting for view! macro

---

## Cloned Repos Status

All repos cloned to `~/pp/_leptos-exploration/`:

| Repo | Status | Priority |
|------|--------|----------|
| `start-axum` | CLONED | P0 |
| `start-axum-workspace` | CLONED | P0 |
| `cargo-leptos` | CLONED | P0 |
| `leptos` (main repo) | CLONED | P0 |
| `leptos-image` | CLONED | P1 |
| `leptos-query` | CLONED | P1 |
| `leptos-use` | CLONED | P0 |
| `thaw` | CLONED | P1 |

### Note: nicoburniske.com

Source code is **NOT PUBLIC**. However, the author (Nico Burniske) created:
- `leptos-image` - which we have cloned
- `leptos-query` - async state management (cloned)
- `tailwind-fuse` - class management

These libraries likely power nicoburniske.com and are available for study.

---

## Parallel Agent Investigation Tasks

All repos cloned. These investigations can run in parallel with haiku agents:

---

## Track B: Compiler-Driven Workflow (COMPLETED)

**Plan Document**: `.claude/workflow-investigation-plan.md`
**Pre-Index**: `.claude/workflow-pre-index.md`
**Status**: ✅ Investigation complete, awaiting synthesis

Investigates integration of cargo-leptos with existing developer workflow:
- Dual-target clippy (ssr + hydrate)
- bacon.toml configuration
- leptosfmt pre-commit integration
- CI pipeline for WASM builds

### Agent B1: Dual-Target Linting
**Scope**: Clippy/tests with feature flags
**Sources**: `leptos/.github/workflows/`, `start-axum/`

### Agent B2: cargo-leptos + bacon Integration
**Scope**: Development workflow tools
**Sources**: `cargo-leptos/src/`, bacon docs (web)

### Agent B3: leptosfmt Integration
**Scope**: view! macro formatting
**Sources**: leptosfmt repo (web), pre-commit patterns

### Agent B4: CI Pipeline Patterns
**Scope**: GitHub Actions for Leptos
**Sources**: `leptos/.github/workflows/`, `cargo-leptos/.github/workflows/`

### Agent B5: cargo-dist + Leptos (CRITICAL)
**Scope**: Can cargo-dist handle dual-artifact Leptos builds?
**Sources**: cargo-dist docs (raw GitHub), cargo-leptos output

### Agent B6: release-plz Workspace
**Scope**: Versioning for CLI + webapp workspace
**Sources**: release-plz docs (raw GitHub)

---

## Track A: Core Framework (COMPLETED)

### Agent 1: Axum Integration Patterns
**Scope**: Server function wiring, state injection, middleware
**Sources**:
- `~/pp/_leptos-exploration/start-axum/`
- `~/pp/_leptos-exploration/start-axum-workspace/`
- `~/pp/_leptos-exploration/leptos/integrations/axum/`
**Key Questions**:
- How to inject custom Axum state (dual S3 clients)?
- Server function → Axum route wiring?
- Middleware integration patterns?
**Output**: `~/pp/_leptos-exploration/REPORT-axum-integration.md`

### Agent 2: Reactivity & State Management
**Scope**: Signals, derived state, async resources
**Sources**:
- `~/pp/_leptos-exploration/leptos-use/`
- `~/pp/_leptos-exploration/leptos-query/`
- Leptos MCP docs: `signals`, `resources`, `actions`
**Key Questions**:
- Signal patterns for undo-able ranking selection?
- Async data fetching for matchups?
- Session state persistence?
**Output**: `~/pp/_leptos-exploration/REPORT-reactivity.md`

### Agent 3: Image Handling Patterns
**Scope**: Progressive loading, optimization, S3 integration
**Sources**:
- `~/pp/_leptos-exploration/leptos-image/`
- leptos_image examples
**Key Questions**:
- Progressive loading (thumb → preview → original)?
- LQIP (Low Quality Image Placeholders)?
- External storage (S3) compatibility?
**Output**: `~/pp/_leptos-exploration/REPORT-image-handling.md`

### Agent 4: Build & Project Structure
**Scope**: cargo-leptos, workspace vs single-crate, Tailwind
**Sources**:
- `~/pp/_leptos-exploration/cargo-leptos/`
- `~/pp/_leptos-exploration/start-axum/Cargo.toml`
- `~/pp/_leptos-exploration/start-axum-workspace/`
**Key Questions**:
- Leptos.toml configuration?
- Workspace structure for CLI + webapp?
- Tailwind build integration?
**Output**: `~/pp/_leptos-exploration/REPORT-build-structure.md`

### Agent 5: Component Architecture
**Scope**: Component patterns, composability, styling
**Sources**:
- `~/pp/_leptos-exploration/thaw/`
- `~/pp/_leptos-exploration/leptos/examples/`
- Leptos MCP docs: `components`, `views`
**Key Questions**:
- Component prop patterns?
- Composable vs monolithic components?
- Dynamic class binding (for gold/silver/bronze)?
**Output**: `~/pp/_leptos-exploration/REPORT-components.md`

---

## Key Architectural Decisions to Make

After investigation, we need answers to:

1. **Single crate or workspace?**
   - Current: single crate webapp
   - Future: CLI + webapp
   - Which template fits better?

2. **Component library or custom?**
   - Thaw vs shadcn vs hand-rolled?
   - Simple Made Easy: which reduces entanglement?

3. **Image optimization strategy**
   - leptos_image integration or external (CLI)?
   - Progressive loading implementation

4. **State management**
   - Signals only or leptos-use utilities?
   - Session state persistence approach

5. **Styling**
   - Tailwind (current direction)
   - Integration with cargo-leptos build

6. **Dual S3 client preservation**
   - How to inject into server functions?
   - Axum state access patterns

---

## Investigation Output Format

Each parallel agent should produce a report with:

```markdown
# [Topic] Investigation Report

## Summary (3-5 sentences)

## Key Findings

### Finding 1: [Title]
- What: ...
- Why it matters for filmorator: ...
- Code example if relevant

### Finding 2: ...

## Recommendations

### Recommended Approach
...

### Alternatives Considered
...

## Constitutional Compliance

### Simple Made Easy
- Does this reduce entanglement? [Yes/No/Partially]
- Evidence: ...

### Correct By Construction
- Compile-time guarantees leveraged? [Yes/No/Partially]
- Evidence: ...

## Files/Locations Referenced
- path/to/file:line - description
```
