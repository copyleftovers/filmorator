# Storage Debugging (MinIO/S3)

## Table of Contents
- [CLI Setup](#cli-setup)
- [Bucket Operations](#bucket-operations)
- [File Operations](#file-operations)
- [Presigned URLs](#presigned-urls)
- [Access Policies](#access-policies)

## CLI Setup

**Configure alias (if not set):**
```bash
mc alias set local http://localhost:9000 minioadmin minioadmin
```

**Verify connection:**
```bash
mc admin info local
```

## Bucket Operations

**List buckets:**
```bash
mc ls local/
```

**List bucket contents (top-level):**
```bash
mc ls local/filmorator/
```
Expected: `original/`, `preview/`, `thumb/`

**List specific tier:**
```bash
mc ls local/filmorator/original/
mc ls local/filmorator/preview/
mc ls local/filmorator/thumb/
```

**Count files:**
```bash
mc ls local/filmorator/original/ | wc -l
```

## File Operations

**Check if file exists:**
```bash
mc stat local/filmorator/original/<filename>
```

**Upload single file:**
```bash
mc cp /path/to/image.jpg local/filmorator/original/
```

**Upload to all tiers:**
```bash
mc cp image.jpg local/filmorator/original/
mc cp image.jpg local/filmorator/preview/
mc cp image.jpg local/filmorator/thumb/
```

**Bulk upload:**
```bash
mc cp --recursive /path/to/images/ local/filmorator/original/
```

**Download file:**
```bash
mc cp local/filmorator/original/<filename> ./
```

**Delete file:**
```bash
mc rm local/filmorator/original/<filename>
```

## Presigned URLs

**How it works:**
1. Browser requests `/img/:tier/:id`
2. App looks up photo by position in DB
3. App generates presigned URL using `presign_client` (configured with `S3_PUBLIC_URL`)
4. App returns 302 redirect to presigned URL
5. Browser fetches image directly from MinIO

**Test redirect:**
```bash
curl -v http://localhost:3000/img/thumb/0 2>&1 | grep -E "(< HTTP|< Location)"
```
Should show: `302` and `Location: http://localhost:9000/filmorator/thumb/...`

**Test presigned URL directly:**
```bash
REDIRECT=$(curl -s -o /dev/null -w '%{redirect_url}' http://localhost:3000/img/thumb/0)
curl -I "$REDIRECT"
```
Should return: `200 OK`, `Content-Type: image/jpeg`

**Common presigned URL issues:**

| Issue | Symptom | Cause |
|-------|---------|-------|
| Wrong host | URL contains `minio:9000` | `S3_PUBLIC_URL` not set |
| 403 Forbidden | Signature mismatch | URL generated with different host than accessed |
| 404 Not Found | File doesn't exist | Check `mc stat` |
| Expired | `Request has expired` | Default expiry is 900s |

## Access Policies

**Check bucket policy:**
```bash
mc anonymous get local/filmorator
```
Should show: `download` (public read)

**Set public read (if missing):**
```bash
mc anonymous set download local/filmorator
```

**MinIO Console:**
- URL: http://localhost:9001
- Credentials: minioadmin / minioadmin
- Can browse buckets, check policies, view files

## S3 Client Architecture

The Rust app has two S3 clients:

```rust
struct S3Client {
    internal_client: Client,  // Uses minio:9000 (Docker network)
    presign_client: Client,   // Uses localhost:9000 (browser-accessible)
}
```

- `internal_client`: Used for `list_objects_v2` (bucket listing)
- `presign_client`: Used for generating presigned URLs

This split is essential because:
- Internal operations need Docker network hostname
- Presigned URLs need browser-accessible hostname
- Using wrong client for presigning → signature mismatch → 403
