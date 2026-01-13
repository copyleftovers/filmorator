# API Debugging

## Table of Contents
- [Endpoint Reference](#endpoint-reference)
- [Testing with Curl](#testing-with-curl)
- [Session Handling](#session-handling)
- [Request/Response Tracing](#requestresponse-tracing)

## Endpoint Reference

| Method | Path | Purpose | Auth |
|--------|------|---------|------|
| GET | `/` | Landing page | None |
| GET | `/compare` | Comparison UI | None |
| POST | `/api/matchup` | Get next photo group | Session |
| POST | `/api/compare` | Submit ranking | Session |
| GET | `/api/ranking` | Get session rankings | Session |
| GET | `/api/progress` | Get completion stats | Session |
| POST | `/api/sync` | Re-sync photos from S3 | None |
| GET | `/img/:tier/:id` | Get presigned image URL | None |

## Testing with Curl

**Basic health check:**
```bash
curl -s http://localhost:3000/api/progress | jq
```

**With session (cookies):**
```bash
# First request creates session, save cookie
curl -s -X POST http://localhost:3000/api/matchup \
  -H "Content-Type: application/json" \
  -c cookies.txt | jq

# Subsequent requests use saved cookie
curl -s http://localhost:3000/api/progress \
  -b cookies.txt | jq
```

**Get matchup:**
```bash
curl -s -X POST http://localhost:3000/api/matchup \
  -H "Content-Type: application/json" \
  -c cookies.txt -b cookies.txt | jq
```

Response:
```json
{
  "matchup_id": "uuid",
  "photos": [
    {"id": 0, "thumb_url": "/img/thumb/0", "preview_url": "/img/preview/0"},
    {"id": 1, "thumb_url": "/img/thumb/1", "preview_url": "/img/preview/1"},
    {"id": 2, "thumb_url": "/img/thumb/2", "preview_url": "/img/preview/2"}
  ]
}
```

**Submit comparison:**
```bash
curl -s -X POST http://localhost:3000/api/compare \
  -H "Content-Type: application/json" \
  -d '{"matchup_id": "<uuid>", "ranked_indices": [0, 2, 1]}' \
  -b cookies.txt | jq
```

**Get rankings:**
```bash
curl -s http://localhost:3000/api/ranking -b cookies.txt | jq
```

**Get progress:**
```bash
curl -s http://localhost:3000/api/progress -b cookies.txt | jq
```

Response:
```json
{
  "compared_pairs": 21,
  "total_pairs": 4950,
  "percent": 0
}
```

**Trigger S3 sync:**
```bash
curl -s -X POST http://localhost:3000/api/sync | jq
```

**Test image redirect:**
```bash
# See redirect
curl -v http://localhost:3000/img/thumb/0 2>&1 | grep -E "(< HTTP|< Location)"

# Get final URL
curl -s -o /dev/null -w '%{redirect_url}' http://localhost:3000/img/thumb/0

# Test full chain
curl -I "$(curl -s -o /dev/null -w '%{redirect_url}' http://localhost:3000/img/thumb/0)"
```

## Session Handling

Sessions are cookie-based. Key behaviors:

1. **New session on first `/api/matchup`** - Creates session, sets cookie
2. **Session persists across requests** - Same cookie = same session
3. **Progress is per-session** - Different sessions have independent progress
4. **Rankings are per-session** - Each user builds their own ranking

**Debug session issues:**
```bash
# Check if cookie is being set
curl -v -X POST http://localhost:3000/api/matchup 2>&1 | grep -i set-cookie

# Verify session exists in DB
docker compose exec db psql -U filmorator -d filmorator \
  -c "SELECT * FROM sessions ORDER BY created_at DESC LIMIT 5"
```

## Request/Response Tracing

**Watch app logs while making requests:**
```bash
# Terminal 1: Follow logs
docker compose logs app -f

# Terminal 2: Make request
curl -s http://localhost:3000/api/progress | jq
```

**Verbose curl output:**
```bash
curl -v http://localhost:3000/api/progress 2>&1
```

Shows:
- Request headers sent
- Response headers received
- Response body

**From browser context (with cookies):**
Use Chrome MCP `javascript_tool`:
```javascript
fetch('/api/progress').then(r => r.json())
```

This uses browser's session cookie, matching what the UI sees.

**Compare CLI vs browser:**
If `curl` and browser `fetch` return different results, it's likely a session/cookie issue.
