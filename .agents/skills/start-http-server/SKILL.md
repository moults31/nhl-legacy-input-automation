---
name: start-http-server
description: >-
  Start a local HTTP server at the repo root so the developer can browse
  screenshots and other repo files in a web browser. Use this when the user
  wants to view screenshots, share local files over HTTP, or inspect repo
  contents through a browser.
---

# start-http-server

Serves the repo root over HTTP using `simple-http-server` (a Rust crate).
Provides browsable directory listings — primarily useful for the `screenshots/`
directory, but all repo content is served.

## Prerequisites

Install once:

```sh
cargo install simple-http-server
```

## Usage

Start the server in the background at the repo root:

```sh
nohup simple-http-server -p 4000 -s . > /dev/null 2>&1 &
```

Open http://localhost:4000/screenshots/ in a browser to browse screenshots.

Flags:
- `-p 4000` — port (default is 8000)
- `-s` — suppress request logs
- `.` — serve the current directory (the repo root)

Binds to `0.0.0.0` (all interfaces) by default. Add `--ip 127.0.0.1` to
restrict to localhost if needed.

## Killing the server

```sh
pkill simple-http-server
```
