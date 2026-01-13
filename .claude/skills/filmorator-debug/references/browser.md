# Browser Debugging (Chrome MCP)

## Table of Contents
- [Setup](#setup)
- [Core Workflow](#core-workflow)
- [Network Request Debugging](#network-request-debugging)
- [Console Messages](#console-messages)
- [Page Interaction](#page-interaction)
- [JavaScript Execution](#javascript-execution)

## Setup

**Get tab context (always first):**
```
mcp__claude-in-chrome__tabs_context_mcp(createIfEmpty: true)
```
Returns: `{availableTabs: [{tabId, title, url}], tabGroupId}`

**Create fresh tab:**
```
mcp__claude-in-chrome__tabs_create_mcp()
```

## Core Workflow

```
1. tabs_context_mcp(createIfEmpty: true)  → get tabId
2. navigate(url, tabId)                    → load page
3. computer(action: "wait", duration: 2)   → let content render
4. computer(action: "screenshot")          → visual verification
5. read_network_requests(tabId)            → check HTTP status codes
```

## Network Request Debugging

**CRITICAL: Lazy-start behavior.** Network tracking starts when `read_network_requests` is first called. Previous requests are not captured.

**Pattern for capturing requests:**
```
1. read_network_requests(tabId)            # Start tracking (returns nothing)
2. navigate(url, tabId)                    # Trigger requests
3. computer(action: "wait", duration: 2)   # Let requests complete
4. read_network_requests(tabId, ...)       # Now read captured requests
```

**Parameters:**
| Parameter | Purpose |
|-----------|---------|
| `tabId` | Required |
| `urlPattern` | Substring filter (e.g., `/api/` or `localhost:9000`) |
| `limit` | Max requests to return |
| `clear` | `true` to empty buffer after reading |

**Response format:**
```json
{
  "url": "http://localhost:3000/api/matchup",
  "method": "POST",
  "statusCode": 200
}
```
`statusCode: "pending"` means still in flight.

**Filtering examples:**
```
urlPattern: "/api/"         → Only API calls
urlPattern: "localhost:9000" → Only MinIO requests
urlPattern: "/img/"         → Only image endpoints
```

## Console Messages

Same lazy-start pattern as network requests.

```
mcp__claude-in-chrome__read_console_messages(
  tabId: <id>,
  pattern: "error",     # Regex filter
  onlyErrors: true,     # Only errors/exceptions
  limit: 20,
  clear: true
)
```

## Page Interaction

**Find elements (natural language):**
```
mcp__claude-in-chrome__find(tabId, query: "submit button")
```
Returns `ref_id` for clicking.

**Read page structure:**
```
mcp__claude-in-chrome__read_page(
  tabId,
  filter: "interactive",  # Only clickable elements
  depth: 5
)
```

**Click element (prefer ref over coordinates):**
```
mcp__claude-in-chrome__computer(
  action: "left_click",
  ref: "ref_5",
  tabId
)
```

**Screenshot:**
```
mcp__claude-in-chrome__computer(action: "screenshot", tabId)
```
Returns screenshot ID (`ss_XXXXX`) for potential `upload_image` use.

**Wait:**
```
mcp__claude-in-chrome__computer(action: "wait", duration: 2, tabId)
```

## JavaScript Execution

**Direct page context execution:**
```
mcp__claude-in-chrome__javascript_tool(
  action: "javascript_exec",
  tabId,
  text: "document.title"
)
```

**Async fetch testing:**
```
text: "fetch('/api/progress').then(r => r.json())"
```
Returns last expression value. Useful for testing APIs from browser context (with cookies).

**Extract page text:**
```
mcp__claude-in-chrome__get_page_text(tabId)
```

## Debugging Patterns

**Image loading failure:**
1. Screenshot → are images showing or broken?
2. `read_network_requests(urlPattern: "/img/")` → check status codes
3. If 200 but no image: check redirect URL in browser devtools
4. If 404: route not matching (check `:param` syntax)
5. If 403: presigned URL host mismatch

**API failure:**
1. `javascript_tool` with fetch → test from browser context
2. Compare with `curl` from CLI → isolate cookie/session issues
3. Check network requests for actual status code

**UI state verification:**
1. Screenshot before action
2. Perform action (click, submit)
3. Wait 1-2 seconds
4. Screenshot after
5. Compare visually
