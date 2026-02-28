FROM ghcr.io/cross-rs/x86_64-unknown-linux-musl:edge

RUN if command -v apk >/dev/null 2>&1; then \
      apk add --no-cache \
        pkgconf \
        sqlite-dev \
      && if [ -d /usr/x86_64-linux-musl/lib ]; then \
           [ -f /usr/lib/libsqlite3.so ] && ln -sf /usr/lib/libsqlite3.so /usr/x86_64-linux-musl/lib/libsqlite3.so || true; \
           [ -f /usr/lib/libsqlite3.a ] && ln -sf /usr/lib/libsqlite3.a /usr/x86_64-linux-musl/lib/libsqlite3.a || true; \
         fi; \
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
ENV PKG_CONFIG_LIBDIR=/usr/lib/pkgconfig:/usr/share/pkgconfig
ENV PKG_CONFIG_PATH=/usr/lib/pkgconfig:/usr/share/pkgconfig
