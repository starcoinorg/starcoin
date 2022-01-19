FROM ubuntu:bionic AS builder

RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        gcc \
        libc6-dev \
	git \
        libssl-dev \
        wget \
        pkg-config \
        libclang-dev clang; \
    rm -rf /var/lib/apt/lists/*

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUSTUP_VERSION=1.22.1 \
    RUSTUP_SHA256=49c96f3f74be82f4752b8bffcf81961dea5e6e94ce1ccba94435f12e871c3bdb \
    RUST_ARCH=x86_64-unknown-linux-gnu

RUN set -eux; \
    url="https://static.rust-lang.org/rustup/archive/${RUSTUP_VERSION}/${RUST_ARCH}/rustup-init"; \
    wget "$url"; \
    echo "${RUSTUP_SHA256} *rustup-init" | sha256sum -c -; \
    chmod +x rustup-init

ENV RUST_VERSION=1.46.0

RUN set -eux; \
    ./rustup-init -y --no-modify-path --default-toolchain $RUST_VERSION; \
    rm rustup-init; \
    chmod -R a+w $RUSTUP_HOME $CARGO_HOME; \
    rustup --version; \
    cargo --version; \
    rustc --version;

WORKDIR /starcoin
COPY ./ .
RUN cargo build --release

FROM ubuntu:bionic
RUN set -eux; \
    apt-get update; \
    apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl-dev; \
	
    rm -rf /var/lib/apt/lists/*

ENV RELEASE_PATH="/starcoin/target/release"
COPY --from=builder $RELEASE_PATH/starcoin \
     $RELEASE_PATH/starcoin_miner \
     $RELEASE_PATH/starcoin_txfactory \
     $RELEASE_PATH/starcoin_faucet \
     $RELEASE_PATH/starcoin_generator \
     $RELEASE_PATH/starcoin_indexer \
     $RELEASE_PATH/mpm \
     $RELEASE_PATH/starcoin_db_exporter \
     /starcoin/
     
CMD ["/starcoin/starcoin"]
