# Database Debugging (PostgreSQL)

## Table of Contents
- [Connection](#connection)
- [Schema Overview](#schema-overview)
- [Common Queries](#common-queries)
- [Data Inspection](#data-inspection)
- [Maintenance](#maintenance)

## Connection

**Connect via docker:**
```bash
docker compose exec db psql -U filmorator -d filmorator
```

**One-liner query:**
```bash
docker compose exec db psql -U filmorator -d filmorator -c "SELECT COUNT(*) FROM photos"
```

## Schema Overview

**List tables:**
```sql
\dt
```

**Describe table:**
```sql
\d photos
\d sessions
\d matchups
\d comparison_results
\d photo_ratings
```

**Key tables:**

| Table | Purpose |
|-------|---------|
| `photos` | Photo metadata (filename, position) |
| `sessions` | User sessions (cookie-based) |
| `matchups` | Generated photo groups for comparison |
| `comparison_results` | User rankings submitted |
| `photo_ratings` | Bradley-Terry strength/uncertainty per session |

## Common Queries

### Photos

```sql
-- Count
SELECT COUNT(*) FROM photos;

-- List with positions
SELECT id, filename, position FROM photos ORDER BY position LIMIT 20;

-- Find by filename pattern
SELECT * FROM photos WHERE filename LIKE '%NORMALIZED%' LIMIT 10;

-- Find by position
SELECT * FROM photos WHERE position = 0;
```

### Sessions

```sql
-- Recent sessions
SELECT id, created_at, last_active_at
FROM sessions
ORDER BY last_active_at DESC LIMIT 10;

-- Session activity summary
SELECT
  s.id,
  s.created_at,
  COUNT(cr.id) as comparisons
FROM sessions s
LEFT JOIN comparison_results cr ON cr.session_id = s.id
GROUP BY s.id
ORDER BY s.created_at DESC;
```

### Matchups

```sql
-- Recent matchups
SELECT id, session_id, photo_indices, is_seed, created_at
FROM matchups
ORDER BY created_at DESC LIMIT 10;

-- Matchups for specific session
SELECT * FROM matchups WHERE session_id = '<uuid>';

-- Count by session
SELECT session_id, COUNT(*) FROM matchups GROUP BY session_id;
```

### Comparison Results

```sql
-- Recent comparisons
SELECT id, matchup_id, session_id, ranked_photo_indices, created_at
FROM comparison_results
ORDER BY created_at DESC LIMIT 10;

-- Comparisons per session
SELECT session_id, COUNT(*) as comparisons
FROM comparison_results
GROUP BY session_id
ORDER BY comparisons DESC;
```

### Photo Ratings (Bradley-Terry)

```sql
-- Rankings for a session (best first)
SELECT
  pr.photo_idx,
  pr.strength,
  pr.uncertainty,
  p.filename
FROM photo_ratings pr
JOIN photos p ON p.position = pr.photo_idx
WHERE pr.session_id = '<uuid>'
ORDER BY pr.strength DESC;

-- Top rated across all sessions
SELECT
  p.filename,
  AVG(pr.strength) as avg_strength,
  COUNT(*) as session_count
FROM photo_ratings pr
JOIN photos p ON p.position = pr.photo_idx
GROUP BY p.filename
ORDER BY avg_strength DESC
LIMIT 20;
```

## Data Inspection

**Progress calculation:**
```sql
-- For n photos, total pairs = n*(n-1)/2
SELECT
  (SELECT COUNT(*) FROM photos) as photo_count,
  (SELECT COUNT(*) FROM photos) * ((SELECT COUNT(*) FROM photos) - 1) / 2 as total_pairs;
```

**Session progress:**
```sql
SELECT
  s.id,
  COUNT(DISTINCT cr.id) as compared_pairs,
  (SELECT COUNT(*) FROM photos) * ((SELECT COUNT(*) FROM photos) - 1) / 2 as total_pairs
FROM sessions s
LEFT JOIN comparison_results cr ON cr.session_id = s.id
GROUP BY s.id;
```

## Maintenance

**Reset all user data (keep photos):**
```sql
TRUNCATE comparison_results, matchups, photo_ratings, sessions CASCADE;
```

**Delete specific session:**
```sql
DELETE FROM comparison_results WHERE session_id = '<uuid>';
DELETE FROM matchups WHERE session_id = '<uuid>';
DELETE FROM photo_ratings WHERE session_id = '<uuid>';
DELETE FROM sessions WHERE id = '<uuid>';
```

**Verify migrations ran:**
```sql
SELECT * FROM _sqlx_migrations ORDER BY installed_on;
```
