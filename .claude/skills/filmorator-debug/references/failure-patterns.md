# Common Failure Patterns

## Table of Contents
- [Images Return 404](#images-return-404)
- [Images Return 403 from S3](#images-return-403-from-s3)
- [No Photos Synced at Startup](#no-photos-synced-at-startup)
- [Session Not Persisting](#session-not-persisting)
- [Container Startup Failure](#container-startup-failure)
- [Database Connection Error](#database-connection-error)
- [UI Shows Stale Data](#ui-shows-stale-data)

---

## Images Return 404

**Symptom:** Network requests to `/img/:tier/:id` return 404 Not Found

**Diagnostic steps:**

1. Check route syntax in `main.rs`:
   ```rust
   // Axum 0.7 - CORRECT
   .route("/img/:tier/:id", get(...))

   // Axum 0.8+ syntax - WRONG for 0.7
   .route("/img/{tier}/{id}", get(...))
   ```

2. Verify photo exists in DB:
   ```bash
   docker compose exec db psql -U filmorator -d filmorator \
     -c "SELECT * FROM photos WHERE position = <id>"
   ```

3. Check app logs for handler errors:
   ```bash
   docker compose logs app --tail=50 | grep -i error
   ```

**Fix:** Update route syntax to match axum version.

---

## Images Return 403 from S3

**Symptom:** Redirect works (302), but MinIO returns 403 Forbidden

**Diagnostic steps:**

1. Check presigned URL hostname:
   ```bash
   curl -s -o /dev/null -w '%{redirect_url}' http://localhost:3000/img/thumb/0
   ```
   - BAD: `http://minio:9000/...` (Docker internal hostname)
   - GOOD: `http://localhost:9000/...` (browser-accessible)

2. Verify `S3_PUBLIC_URL` is set:
   ```bash
   docker compose exec app printenv S3_PUBLIC_URL
   ```

3. Check bucket policy:
   ```bash
   mc anonymous get local/filmorator
   ```

**Root cause:** Presigned URL signature is tied to the hostname. URL signed for `minio:9000` is invalid when accessed via `localhost:9000`.

**Fix:** Set `S3_PUBLIC_URL=http://localhost:9000` in docker-compose.yml.

---

## No Photos Synced at Startup

**Symptom:** `/api/progress` shows 0 total_pairs, DB has no photos

**Diagnostic steps:**

1. Check S3 has images:
   ```bash
   mc ls local/filmorator/original/ | head
   ```

2. Check app startup logs:
   ```bash
   docker compose logs app | grep -i "photo\|sync\|s3"
   ```
   Should show: `Found N photos in S3`

3. Verify bucket name matches:
   ```bash
   docker compose exec app printenv FILMORATOR_BUCKET
   # Should match bucket name in MinIO
   ```

4. Check DB directly:
   ```bash
   docker compose exec db psql -U filmorator -d filmorator \
     -c "SELECT COUNT(*) FROM photos"
   ```

**Fix:** Upload images to `original/` prefix, restart app to trigger sync.

---

## Session Not Persisting

**Symptom:** Progress resets between requests, rankings don't accumulate

**Diagnostic steps:**

1. Check curl is using cookies:
   ```bash
   # WRONG - no cookie handling
   curl http://localhost:3000/api/progress

   # RIGHT - save and reuse cookies
   curl -c cookies.txt -b cookies.txt http://localhost:3000/api/progress
   ```

2. Verify sessions in DB:
   ```bash
   docker compose exec db psql -U filmorator -d filmorator \
     -c "SELECT * FROM sessions ORDER BY created_at DESC LIMIT 5"
   ```

3. Check browser cookies:
   - DevTools → Application → Cookies
   - Look for session cookie on `localhost:3000`

**Fix:** Ensure cookie handling in requests. For browser, cookies should be automatic.

---

## Container Startup Failure

**Symptom:** `docker compose up` fails or container exits immediately

**Diagnostic steps:**

1. Check container status:
   ```bash
   docker compose ps
   ```

2. Check exit logs:
   ```bash
   docker compose logs <service> --tail=100
   ```

**Common causes:**

| Error | Cause | Fix |
|-------|-------|-----|
| `edition2024 requires Rust 1.85` | Old Rust in Dockerfile | Update to `rust:1.85-slim` |
| `DATABASE_URL required` | Missing env var | Check docker-compose.yml |
| `connection refused` | DB not ready | Check depends_on + healthcheck |
| `bucket not found` | minio-setup didn't run | Check minio-setup logs |

---

## Database Connection Error

**Symptom:** App logs show connection refused or timeout to database

**Diagnostic steps:**

1. Is DB container running?
   ```bash
   docker compose ps db
   ```

2. Is DB healthy?
   ```bash
   docker compose exec db pg_isready -U filmorator
   ```

3. Can app reach DB?
   ```bash
   docker compose exec app pg_isready -h db -U filmorator
   ```

4. Check DATABASE_URL uses Docker hostname:
   ```bash
   docker compose exec app printenv DATABASE_URL
   # Should contain @db:5432, not @localhost:5432
   ```

**Fix:** Ensure DATABASE_URL uses `db` hostname, DB is healthy before app starts.

---

## UI Shows Stale Data

**Symptom:** Browser shows old photos or wrong progress after changes

**Diagnostic steps:**

1. Hard refresh browser: `Cmd+Shift+R` or `Ctrl+Shift+R`

2. Check API returns fresh data:
   ```bash
   curl -s http://localhost:3000/api/progress -b cookies.txt | jq
   ```

3. Compare browser fetch with curl:
   - Use Chrome `javascript_tool`: `fetch('/api/progress').then(r => r.json())`
   - Should match curl output with same session

4. Check for caching headers:
   ```bash
   curl -v http://localhost:3000/api/progress 2>&1 | grep -i cache
   ```

**Fix:** Usually a browser cache issue. Hard refresh or clear cookies.

---

## Debugging Strategy

When encountering unknown issues:

1. **Identify layer:** Is it browser, API, database, or storage?
2. **Reproduce minimally:** Can you trigger with curl? With fresh session?
3. **Check logs:** `docker compose logs app -f` while reproducing
4. **Binary search:** What's the last known good state? What changed?
5. **Isolate:** Does fresh container/data fix it?
