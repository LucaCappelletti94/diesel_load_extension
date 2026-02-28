FROM ghcr.io/cross-rs/x86_64-unknown-linux-musl:edge

# Scaffold image for future stronger system-SQLite checks on x86_64-musl.
# The current CI gate uses cargo check; migrate to cross link/no-run once
# a stable sqlite-dev package path is pinned for this image lineage.
RUN if command -v apk >/dev/null 2>&1; then \
      apk add --no-cache pkgconf; \
    elif command -v apt-get >/dev/null 2>&1; then \
      apt-get update \
      && apt-get install -y --no-install-recommends pkg-config \
      && rm -rf /var/lib/apt/lists/*; \
    else \
      echo "Unsupported package manager" && exit 1; \
    fi

ENV PKG_CONFIG_ALLOW_CROSS=1
