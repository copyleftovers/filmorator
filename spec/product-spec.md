# Filmorator Product Specification

Stakeholder decisions captured 2026-01-14. This document is the source of truth for WHAT we're building. VISION.md describes HOW.

---

## Product Identity

**What it is:** Crowdsourced photo ranking for solo photographers sharing with their network.

**What it is NOT:**
- A public platform (no discovery, no directory)
- A social network (no accounts, no follows)
- A professional tool (no clients, no invoicing)

---

## User Model

### Campaign Owner

| Aspect | Decision |
|--------|----------|
| Identity | No accounts. URL-based management. Lose URL = lose campaign. |
| Capabilities | Create campaign (CLI), share link, view results, view contributors, close campaign |
| Visibility | Sees full contributor breakdown: names (if provided) + comparison counts |

### Participant

| Aspect | Decision |
|--------|----------|
| Identity | Session-based (cookie). Optional name field (always visible, fill anytime). |
| Discovery | Link-only. Owner shares URL directly. |
| Motivation | Social obligation (helping friends) + curiosity (seeing results) |
| Visibility | Sees other participants' names if they provided them |

---

## Campaign Lifecycle

```
CREATE (CLI)
    ↓
SHARE (URL)
    ↓
COLLECT (comparisons accumulate)
    ↓
SOFT THRESHOLD (owner notified, collection continues)
    ↓
CLOSE (owner action)
    ↓
ARCHIVE (read-only, results visible)
    ↓
REOPEN? (owner can gather more data)
```

### Mutability
- **Immutable after creation.** No adding/removing photos once campaign exists.
- Rationale: Ranking integrity. Changing photos invalidates existing comparisons.

### Completion
- **Soft threshold:** System notifies owner when statistically meaningful, but keeps collecting.
- **Manual close:** Owner explicitly ends collection.
- **Reopenable:** Can resume collection later for higher confidence.

---

## Incentive Design

### Participant Feedback (ALL in MVP)

1. **Contribution count:** "You've ranked 12 matchups"
2. **Campaign progress:** "Campaign is 45% complete"
3. **Impact feedback:** "Your comparison moved Photo 7 up 2 positions"
4. **Personal vs aggregate:** Show how participant's rankings differ from crowd

### Results Access

- **Unlock condition:** After threshold of comparisons where aggregate is statistically meaningful
- **Final reveal:** Full results visible after owner closes campaign
- **Dual incentive:** Contribute to unlock preview, return later to see final

### Social Layer

- Participants see: "Alice, Bob, and 21 others contributed"
- Owner sees: Full breakdown with names and counts
- Names are optional, low-friction, changeable anytime

---

## Scale & Constraints

| Constraint | Value | Rationale |
|------------|-------|-----------|
| Photos per campaign | 50-200 | Film roll to full shoot |
| Image format | JPEG only | Film scans are JPEG |
| Privacy | Link-only (no public directory) | Close network tool |
| Data retention | Indefinite until owner deletes | Simple, no expiration logic |

---

## Error Handling

| Scenario | Behavior |
|----------|----------|
| Upload fails | Fail loudly, require retry |
| DB error | Fail loudly, require retry |
| Suspected abuse | Flag for owner, owner decides exclusion |
| Partial operations | No silent failures, no auto-recovery |

---

## Success Criteria

### MVP Definition
Core flow + full engagement system:
- Create campaign (CLI)
- Share link
- Rank photos (with all feedback types)
- See results (with unlock mechanics)
- Optional names
- Personal vs aggregate comparison

### Success Metrics
- Personal use: Creator uses it for own film ranking
- Friend adoption: People shared with actually complete campaigns
- Social vetting: Strangers OK if referred through network

---

## Clarified Decisions

| Question | Answer |
|----------|--------|
| Threshold notification | Poll-based. Owner checks management URL. No email/webhook. |
| Abuse detection | Speed-based + pattern-based + manual owner flagging |
| Campaign deletion | Not supported. Campaigns persist forever. Close/reopen only. |

---

## Deferred to Post-MVP

1. **QR code generation** - Useful for physical sharing
2. **Embed widget** - Blog/portfolio embedding
3. **Gamification** - Points/streaks, only if organic engagement insufficient
4. **Account system** - If multi-campaign management becomes painful
5. **Active notifications** - Email/webhook when threshold reached

---

## Relationship to VISION.md

| This Document (WHAT) | VISION.md (HOW) |
|----------------------|-----------------|
| Immutable campaigns | S3 manifest pattern |
| Session-based participants | Cookie + DB sessions table |
| Statistical unlock threshold | Bradley-Terry confidence interval |
| Optional names | Session metadata column |
| JPEG only | CLI validation step |
| Fail loudly | No retry middleware |
