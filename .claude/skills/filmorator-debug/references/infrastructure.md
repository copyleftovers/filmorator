# Infrastructure Debugging (Docker)

## Table of Contents
- [Container Status](#container-status)
- [Service Logs](#service-logs)
- [Container Management](#container-management)
- [Network Debugging](#network-debugging)
- [Common Issues](#common-issues)

## Container Status

```bash
docker compose ps
```

Expected: All services show `Up` and `healthy`

Services in this stack:
- `db` - PostgreSQL 16
- `minio` - S3-compatible storage
- `minio-setup` - One-shot bucket initialization
- `app` - Filmorator Rust application

## Service Logs

**Recent logs:**
```bash
docker compose logs <service> --tail=50
```

**Follow logs (live):**
```bash
docker compose logs <service> -f
```

**All services:**
```bash
docker compose logs --tail=20
```

**Filter for errors:**
```bash
docker compose logs app 2>&1 | grep -i error
```

## Container Management

**Restart single service:**
```bash
docker compose restart app
```

**Rebuild and restart:**
```bash
docker compose up --build -d app
```

**Full teardown:**
```bash
docker compose down
```

**Teardown with volume removal (data loss!):**
```bash
docker compose down -v
```

**Fresh start:**
```bash
docker compose down && docker compose up -d
```

## Network Debugging

**Test internal DNS resolution:**
```bash
docker compose exec app ping -c1 minio
docker compose exec app ping -c1 db
```

**Check exposed ports:**
```bash
docker compose ps --format "table {{.Name}}\t{{.Ports}}"
```

Expected ports:
- `3000` - App (Axum)
- `5432` - PostgreSQL
- `9000` - MinIO API
- `9001` - MinIO Console

## Common Issues

### App exits immediately

**Check:** `docker compose logs app --tail=50`

**Causes:**
1. Missing env vars → config parse error
2. DB not ready → connection refused (should be handled by healthcheck)
3. Rust version too old → needs 1.85+

### DB connection refused

**Check:** `docker compose ps db` - is it healthy?

**Test connection:**
```bash
docker compose exec app pg_isready -h db -U filmorator
```

### MinIO not accessible

**Check:** `docker compose ps minio`

**Test from app container:**
```bash
docker compose exec app curl -I http://minio:9000/minio/health/live
```

### Service dependency issues

Services start order defined by `depends_on` + healthchecks:
1. `db` starts, waits for healthy
2. `minio` starts, waits for healthy
3. `minio-setup` runs (creates bucket)
4. `app` starts after setup completes

If order breaks, try: `docker compose up -d --wait`
