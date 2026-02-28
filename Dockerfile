# =====================================================
# Stage 1: Builder — Compile on Linux (+ Windows cross)
# =====================================================
FROM rust:1.88-bookworm AS builder

WORKDIR /build

# Install Windows cross-compilation toolchain
RUN apt-get update \
    && apt-get install -y --no-install-recommends gcc-mingw-w64-x86-64 \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add x86_64-pc-windows-gnu

# Layer-Cache: Dependencies first, source second
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs && echo "" > src/lib.rs \
    && cargo build --release 2>/dev/null || true \
    && rm -rf src target/release/rush-sync target/release/deps/rush_sync* target/release/deps/libr*

# Full source copy + real build (Linux)
COPY src/ src/
RUN touch src/main.rs src/lib.rs && cargo build --release

# Windows cross-compile
RUN cargo build --release --target x86_64-pc-windows-gnu 2>/dev/null || echo "Windows cross-compile skipped"

# Prepare downloads directory
RUN mkdir -p /build/downloads \
    && cp /build/target/release/rush-sync /build/downloads/rush-sync-linux-amd64.bin \
    && (cp /build/target/x86_64-pc-windows-gnu/release/rush-sync.exe /build/downloads/rush-sync-windows-amd64.exe 2>/dev/null || true)

# =====================================================
# Stage 2: Runtime — Slim Debian image
# =====================================================
FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary into WORKDIR — config is resolved relative to the binary path
COPY --from=builder /build/target/release/rush-sync /app/rush-sync

# Copy downloads (Linux from builder, macOS from local project)
COPY --from=builder /build/downloads/ /usr/local/share/downloads/
COPY downloads/rush-sync-macos-arm64.bin /usr/local/share/downloads/rush-sync-macos-arm64.bin

# Copy entrypoint script and landing page
COPY docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh
RUN chmod +x /usr/local/bin/docker-entrypoint.sh
COPY landing.html /usr/local/share/landing.html
COPY downloads/startup.mp4 /usr/local/share/downloads/startup.mp4
COPY downloads/startup.webm /usr/local/share/downloads/startup.webm
COPY downloads/README.md /usr/local/share/downloads/README.md
COPY README.md /usr/local/share/README.md
COPY STARTUP.md /usr/local/share/STARTUP.md
COPY BENCHMARK.md /usr/local/share/BENCHMARK.md

# Server + Proxy ports
EXPOSE 80 8000 8001 3000 3443

ENTRYPOINT ["docker-entrypoint.sh"]
