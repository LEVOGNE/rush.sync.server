# Development Directory · {{SERVER_NAME}}

This is the developer workspace for **Rush Sync Server**.

## Quick Start

1. Put your HTML/CSS/JS files in this folder.
2. **Hot-Reload is always on** – no script tag needed (the server injects automatically).
3. Your main entry is **`index.html`** at `/`.
   If `index.html` is missing, the server redirects `/` → `/.rss/`.

## Server Info

- **Server Name:** `{{SERVER_NAME}}`
- **Port:** `{{PORT}}`
- **Server URL:** `http://127.0.0.1:{{PORT}}`
- **Dashboard (embedded):** `http://127.0.0.1:{{PORT}}/admin`
- **Hot-Reload WS:** `ws://127.0.0.1:{{PORT}}/ws/hot-reload`

## Features

- **Automatic Hot-Reload** (server-side injection of `/rss.js`)
- **Template Variables** (`{{SERVER_NAME}}`, `{{PORT}}`) replaced on the fly
- **Security Monitoring** & **Live Metrics** in `/admin`
- **SEO Ready** (`robots.txt` supported if present)

## File Structure

```text
{{SERVER_NAME}}-[{{PORT}}]/
├─ README.md        # this file
└─robots.txt       # SEO config (optional)
```

## Development Workflow

1. Create `index.html` for your main page
2. Add CSS files for styling
3. Add JavaScript for interactivity
4. Files are watched automatically
5. Browser reloads on any change

## System URLs

- `/.rss/` - Rush Sync Dashboard
- `/rss.js` - Hot-Reload System (automatically injected)
- `/api/*` - Server API endpoints
- `/ws/hot-reload` - WebSocket for live updates

Happy coding!
