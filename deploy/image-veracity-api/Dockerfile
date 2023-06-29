# syntax=docker/dockerfile:1.3-labs
# Run from root context ".":
# docker build -f deploy/image-veracity-api/Dockerfile -t veracity-project/image-veracity-api .

FROM rustlang/rust:nightly AS build

# Capture dependencies
COPY Cargo.toml Cargo.lock /app/

# We create a new lib and then use our own Cargo.toml
RUN cargo new --lib /app/crates/trillian && \
    cargo new --lib /app/crates/smt
COPY crates/trillian/Cargo.toml /api/crates/trillian/
COPY crates/smt/Cargo.toml /api/crates/smt/


# We do the same for our app
RUN cargo new /app/crates/image-veracity-api
COPY crates/image-veracity-api/Cargo.toml /app/crates/image-veracity-api/

# This step compiles only our dependencies and saves them in a layer. This is the most impactful time savings
# Note the use of --mount=type=cache. On subsequent runs, we'll have the crates already downloaded
RUN --mount=type=cache,target=/usr/local/cargo/registry cargo +nightly build --manifest-path /app/Cargo.toml --release

# Copy our sources
COPY crates/image-veracity-api /app/crates/image-veracity-api
COPY crates/trillian /app/crates/trillian
COPY crates/smt /app/crates/smt

# A bit of magic here!
# * We're mounting that cache again to use during the build, otherwise it's not present and we'll have to download those again - bad!
# * EOF syntax is neat but not without its drawbacks. We need to `set -e`, otherwise a failing command is going to continue on
# * Rust here is a bit fiddly, so we'll touch the files (even though we copied over them) to force a new build
RUN --mount=type=cache,target=/usr/local/cargo/registry <<EOF
  set -e
  # update timestamps to force a new build
  touch /app/crates/trillian/src/lib.rs /app/crates/image-veracity-api/src/main.rs
  cargo +nightly build --manifest-path /app/crates/image-veracity-api/Cargo.toml --release
EOF

CMD ["/app/target/release/image-veracity-api"]

# Again, our final image is the same - a slim base and just our app
FROM rust:1.70 AS app
COPY --from=build /app/target/release/image-veracity-api /image-veracity-api
COPY resources/crdb/root.crt /root.crt
CMD ["/image-veracity-api"]