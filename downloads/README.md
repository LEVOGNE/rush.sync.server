# Rush Sync Server — Installation

## Linux

```bash
chmod +x rush-sync
./rush-sync
```

## macOS

macOS blocks unsigned binaries downloaded from the internet.
After downloading, run these commands:

```bash
xattr -d com.apple.quarantine rush-sync
chmod +x rush-sync
./rush-sync
```

Or: Right-click the file in Finder > "Open" > "Open Anyway".
Or: System Settings > Privacy & Security > "Allow Anyway".

## Windows

Double-click `rush-sync.exe` to run.
If Windows Defender shows a warning: "More info" > "Run anyway".

## Docker

```bash
docker run -p 8000:8000 -p 3000:3000 -p 3443:3443 rush-sync-server
```

## First Start

On first launch, Rush Sync Server creates a `rush.toml` config file
and a `www/` directory for your websites. Run `help` to see all commands.

## Links

- Website: https://rush-sync-server.com
- GitHub: https://github.com/LEVOGNE/rush.sync.server
- Contact: l.ersen@icloud.com

---
v0.3.8 — First Official Public Release
v0.1.0 – v0.3.7: Internal development builds, developed and tested daily since February 2025.
(c) 2025 Levogne (Levent Ersen)
