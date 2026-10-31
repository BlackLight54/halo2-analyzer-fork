# Runtime image for the results map:
#   docker build -t halo2-analyzer-fork .
#   docker run --rm -v "$PWD/out:/out" halo2-analyzer-fork reproduce
# Entry point is `just`, so any recipe name works as the command (smoke, bench-invoice, ...).

# cvc5 with finite-field support (CoCoA, hence --gpl), pinned to the commit the docs/ measurements
# report as `1.3.5.dev+main@1e05d08b9`.
FROM ubuntu:24.04 AS cvc5
ARG CVC5_COMMIT=1e05d08b9f73281473dd06285bcafbeca0ae1b9f
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates git build-essential cmake m4 python3 python3-venv python3-pip libgmp-dev patch curl \
    && rm -rf /var/lib/apt/lists/*
RUN git clone https://github.com/cvc5/cvc5 /src/cvc5 && cd /src/cvc5 && git checkout "$CVC5_COMMIT"
WORKDIR /src/cvc5
RUN ./configure.sh production --cocoa --gpl --auto-download --prefix=/opt/cvc5 \
    && cd build && make -j"$(nproc)" install

FROM ubuntu:24.04
ARG RUST_TOOLCHAIN=nightly-2026-08-28
ARG JUST_VERSION=1.36.0
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates curl build-essential pkg-config git libgmp10 hyperfine \
    && rm -rf /var/lib/apt/lists/* \
    && curl -sSfL https://just.systems/install.sh | bash -s -- --tag "$JUST_VERSION" --to /usr/local/bin
COPY --from=cvc5 /opt/cvc5 /opt/cvc5
ENV RUSTUP_HOME=/opt/rustup CARGO_HOME=/opt/cargo RUSTUP_TOOLCHAIN=$RUST_TOOLCHAIN
ENV PATH=/opt/cvc5/bin:/opt/cargo/bin:$PATH
RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain "$RUST_TOOLCHAIN"
WORKDIR /work
COPY . .
# Compile once at build time so `reproduce` runs offline against the committed Cargo.lock.
RUN cargo build --release --locked -p korrekt --lib --tests
ENV OUT=/out
ENTRYPOINT ["just"]
CMD ["reproduce"]
