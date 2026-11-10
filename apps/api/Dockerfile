FROM rust:1-bookworm AS toolchain
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

FROM toolchain AS build
COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release
FROM debian:bookworm-slim AS release
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/* && useradd --system app
COPY --from=build /app/target/release/purrsonnel-api /usr/local/bin/
USER app
EXPOSE 3000
HEALTHCHECK --interval=15s --timeout=5s --start-period=90s --retries=5 CMD curl --fail --silent http://127.0.0.1:3000/health/ready || exit 1
CMD ["purrsonnel-api"]
