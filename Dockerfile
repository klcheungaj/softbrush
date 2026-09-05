# syntax=docker/dockerfile:1

ARG RUST_VERSION=1.95.0
ARG ALPINE_VERSION=3.22

# GNU development image. This is the container target for building, testing,
# linting, and parser work on glibc-based development systems.
FROM rust:${RUST_VERSION}-bookworm AS gnu-dev

RUN apt-get update && \
    apt-get install --yes --no-install-recommends build-essential && \
    rm -rf /var/lib/apt/lists/* && \
    rustup component add clippy rustfmt

ARG UID=1000
ARG GID=1000
RUN groupadd --gid "$GID" builder && \
    useradd --uid "$UID" --gid "$GID" --create-home --shell /bin/bash builder

WORKDIR /workspace

# Copy manifests first so Cargo's registry downloads remain cached when only
# project source files change.
COPY --chown=builder:builder Cargo.toml Cargo.lock ./
COPY --chown=builder:builder .cargo .cargo
COPY --chown=builder:builder tools/parser-generator/Cargo.toml tools/parser-generator/Cargo.lock tools/parser-generator/

RUN mkdir -p target && chown builder:builder target

ENV CARGO_HOME=/home/builder/.cargo

USER builder
RUN cargo fetch --locked --target x86_64-unknown-linux-gnu && \
    cargo fetch --locked --manifest-path tools/parser-generator/Cargo.toml

COPY --chown=builder:builder grammar grammar
COPY --chown=builder:builder scripts scripts
COPY --chown=builder:builder src src
COPY --chown=builder:builder tests tests
COPY --chown=builder:builder tools/parser-generator/src tools/parser-generator/src
COPY --chown=builder:builder LICENSE README.md ./

CMD ["cargo", "test", "--locked", "--all-targets", "--target", "x86_64-unknown-linux-gnu"]

# Optional GNU release artifact. Development and test builds should normally
# use the gnu-dev stage above.
FROM gnu-dev AS gnu-builder
RUN cargo build --locked --release --target x86_64-unknown-linux-gnu

FROM scratch AS gnu-artifact
COPY --from=gnu-builder /workspace/target/x86_64-unknown-linux-gnu/release/softbrush_ls /softbrush_ls

# Official release builder. Alpine's native GCC targets musl; build-base and
# linux-headers also compile the C implementation bundled by libmimalloc-sys.
FROM rust:${RUST_VERSION}-alpine${ALPINE_VERSION} AS musl-builder

RUN apk add --no-cache \
        bash \
        build-base \
        linux-headers

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /workspace

COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
RUN cargo fetch --locked --target x86_64-unknown-linux-musl

COPY scripts/build-musl.sh scripts/verify-musl.sh scripts/
COPY src src

RUN MUSL_CC=cc ./scripts/build-musl.sh

# An official release must be static and must include mimalloc.
RUN ./scripts/verify-musl.sh target/x86_64-unknown-linux-musl/release/softbrush_ls

FROM scratch AS musl-artifact
COPY --from=musl-builder /workspace/target/x86_64-unknown-linux-musl/release/softbrush_ls /softbrush_ls

# Backward-compatible alias for the original artifact target.
FROM musl-artifact AS artifact

# The default image is the official musl release in a minimal Alpine runtime.
FROM alpine:${ALPINE_VERSION} AS runtime

RUN addgroup -S softbrush && adduser -S -G softbrush softbrush

COPY --from=musl-builder /workspace/target/x86_64-unknown-linux-musl/release/softbrush_ls /usr/local/bin/softbrush_ls

USER softbrush
ENTRYPOINT ["/usr/local/bin/softbrush_ls"]
