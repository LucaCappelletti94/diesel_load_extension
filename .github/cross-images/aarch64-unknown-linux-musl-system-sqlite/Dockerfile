FROM ghcr.io/cross-rs/aarch64-unknown-linux-musl:edge

# Note: this image installs common sqlite development packages, but current
# CI still lacks a reliably linkable target `-lsqlite3` for system mode.
# Keep as scaffolding for future musl system-ABI enablement.
RUN if command -v apk >/dev/null 2>&1; then \
      apk add --no-cache \
        pkgconf \
        sqlite-dev; \
    elif command -v apt-get >/dev/null 2>&1; then \
      apt-get update \
      && apt-get install -y --no-install-recommends \
        pkg-config \
        libsqlite3-dev \
      && rm -rf /var/lib/apt/lists/*; \
    else \
      echo "Unsupported package manager" && exit 1; \
    fi

ENV PKG_CONFIG_ALLOW_CROSS=1
