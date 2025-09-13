FROM rust:latest AS builder

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release --package migration
RUN cargo build --release --package app

FROM rust:latest

RUN useradd -m node

COPY --from=builder /usr/src/app/target/release /usr/node
COPY --from=builder /usr/src/app/resources /usr/node/resources

USER node

EXPOSE 7012

WORKDIR /usr/node
CMD ["sh", "-c", "./migration && ./app"]