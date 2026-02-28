FROM ghcr.io/cross-rs/aarch64-unknown-linux-musl:edge

# Scaffold image for future stronger system-SQLite checks on aarch64-musl.
# The current CI gate uses cargo check; migrate to cross link/no-run once
# a target sysroot SQLite package is pinned and validated.
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
