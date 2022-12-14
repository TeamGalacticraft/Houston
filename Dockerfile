FROM rust:1.65 as build

WORKDIR /usr/src
RUN USER=root cargo new --bin houston
WORKDIR /usr/src/houston

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN echo "fn main() {}" >> build.rs && cargo build --release && rm src/*.rs && rm build.rs

COPY . .
ARG SQLX_OFFLINE=true
RUN cargo build --release


ADD https://github.com/ufoscout/docker-compose-wait/releases/download/2.9.0/wait /wait
RUN chmod +x /wait

FROM debian:bullseye-slim

COPY --from=build /usr/src/houston/target/release/houston /houston/houston
COPY --from=build /usr/src/houston/migrations/* /houston/migrations/
COPY --from=build /wait /wait

CMD /wait && /houston/houston