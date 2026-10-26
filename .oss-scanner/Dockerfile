# Builds Ely for Anthropic's OSS Scanner: the library and its tests, all crates fetched while online.
FROM debian:bookworm
ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    CARGO_NET_OFFLINE=false
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates curl build-essential clang cmake pkg-config git python3 \
      libfontconfig-dev libfreetype-dev libxkbcommon-dev libxkbcommon-x11-dev \
      libwayland-dev libx11-dev libx11-xcb-dev libxcb1-dev libxi-dev libxcursor-dev libxrandr-dev \
      libvulkan-dev libasound2-dev libssl-dev libzstd-dev \
    && rm -rf /var/lib/apt/lists/*
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.98

# Tests check file permissions, which root ignores, so the build runs as a plain user.
RUN useradd --create-home ely
COPY --chown=ely:ely . /src
RUN chown -R ely:ely /usr/local/cargo /usr/local/rustup
USER ely
WORKDIR /src

# Fetch every crate, then build the library and its tests with debug info; tests run offline later.
RUN cargo fetch --locked
RUN cargo build --locked --lib --features test-support \
    && cargo test --locked --lib --features test-support --no-run
ENV CARGO_NET_OFFLINE=true
RUN cargo test --locked --lib --features test-support --quiet || echo "WARNING: cargo test reported failures"
