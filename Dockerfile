FROM rust:1.82-slim-bullseye as builder

WORKDIR /usr/src/app
COPY . .

RUN apt update && apt install pkg-config libssl-dev -y
RUN cargo build --release

RUN cp target/release/cosmos-upgrade-watcher /cosmos-upgrade-watcher

FROM rust:1.82-slim-bullseye
WORKDIR /usr/src/app
COPY --from=builder /cosmos-upgrade-watcher /usr/bin/cosmos-upgrade-watcher

ENTRYPOINT ["/usr/bin/cosmos-upgrade-watcher"]
