# Filmorator Vision

Crowdsource photo rankings. Multiple anonymous participants contribute pairwise comparisons to derive a single global ranking via Bradley-Terry model.

## Core Concept

**The problem:** You have N photos and want to rank them by quality/preference. Doing this alone is biased. Asking others to rank all N is tedious.

**The solution:** Show 3 photos at a time. Ask: "Rank these best to worst." Aggregate many such comparisons from different people into one global ranking using Bradley-Terry maximum likelihood estimation.

**Why it works:** Each 3-way comparison yields 3 pairwise relationships. With enough comparisons covering all pairs transitively (via snic GBER decomposition), we get a statistically valid global ranking.

---

## Documentation Structure

| Document | Focus | Question Answered |
|----------|-------|-------------------|
| `spec/product-spec.md` | Product decisions | WHAT are we building? |
| `spec/personas.md` | User model | WHO are we building for? |
| `spec/user-stories.md` | Requirements | WHAT must it do? |
| `spec/ARCHITECTURE_PLAN.md` | Technical architecture | HOW do we build it? |
| `spec/IMPLEMENTATION_PLAN.md` | Incremental delivery | WHEN can stakeholders test? |

---

## Quick Reference

### Architecture

```
CLI → S3 (immutable manifest + images)
       ↓
Webapp → PostgreSQL (status, sessions, comparisons, ratings)
```

- **CLI writes to S3** (once, at creation)
- **Webapp reads from S3** (never writes)
- **Webapp writes to DB** (status, comparisons)
- **Status lives in DB** (not S3 manifest)

### Key Decisions

| Decision | Choice |
|----------|--------|
| Matchup size | Fixed at 3 |
| Owner access | Secret management URL |
| Accounts | None (sessions only) |
| Campaign deletion | Not supported |
| Personal rankings | Derived on-demand |

### Tech Stack

- **Frontend**: Leptos (Rust, compile-time type safety)
- **Backend**: Axum (wrapped by Leptos)
- **Database**: PostgreSQL
- **Storage**: S3/MinIO
- **Matchups**: snic GBER decomposition
- **Ranking**: Bradley-Terry MLE

---

## References

- **Leptos**: [leptos.dev](https://leptos.dev)
- **Bradley-Terry**: [Wikipedia](https://en.wikipedia.org/wiki/Bradley%E2%80%93Terry_model)
- **gallery-rs**: Reference architecture (`~/pp/gallery-rs`)
- **snic-rs**: Matchup generation (`~/pp/snic-rs`)
