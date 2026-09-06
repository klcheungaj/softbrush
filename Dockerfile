# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.95.0
ARG ALPINE_VERSION=3.22

# Official release builder. Alpine's native GCC targets musl; build-base and
# linux-headers also compile the C implementation bundled by libmimalloc-sys.
FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS musl-builder

ARG TARGETARCH

RUN apk add --no-cache \
        bash \
        build-base \
        linux-headers

RUN case "$TARGETARCH" in \
      amd64) target=x86_64-unknown-linux-musl ;; \
      arm64) target=aarch64-unknown-linux-musl ;; \
      *) echo "unsupported Docker architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac && \
    rustup target add "$target"

WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
RUN case "$TARGETARCH" in \
      amd64) target=x86_64-unknown-linux-musl ;; \
      arm64) target=aarch64-unknown-linux-musl ;; \
      *) echo "unsupported Docker architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac && \
    cargo fetch --locked --target "$target"

COPY scripts/build-linux.sh scripts/verify-musl.sh scripts/
COPY src src

RUN case "$TARGETARCH" in \
      amd64) target=x86_64-unknown-linux-musl ;; \
      arm64) target=aarch64-unknown-linux-musl ;; \
      *) echo "unsupported Docker architecture: $TARGETARCH" >&2; exit 1 ;; \
    esac && \
    MUSL_CC=cc ./scripts/build-linux.sh --arch "${target%%-*}" && \
    mkdir -p /out && \
    install -m 755 "target/$target/release/softbrush_ls" /out/softbrush_ls

# An official release must be static and must include mimalloc.
RUN ./scripts/verify-musl.sh /out/softbrush_ls

FROM scratch AS musl-artifact
COPY --from=musl-builder /out/softbrush_ls /softbrush_ls

# Backward-compatible alias for the original artifact target.
FROM musl-artifact AS artifact

# The default image is the official musl release in a minimal Alpine runtime.
FROM alpine:${ALPINE_VERSION} AS runtime

RUN addgroup -S softbrush && adduser -S -G softbrush softbrush

COPY --from=musl-builder /out/softbrush_ls /usr/local/bin/softbrush_ls

USER softbrush
ENTRYPOINT ["/usr/local/bin/softbrush_ls"]
