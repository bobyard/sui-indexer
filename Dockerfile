FROM rust:latest as build

WORKDIR /usr/src/sui-indexer

COPY . .

RUN cargo build --release

FROM gcr.io/distroless/cc-debian10

COPY --from=build /usr/src/sui-indexer/target/release/sui-index /usr/local/bin/sui-index

WORKDIR /usr/local/bin

CMD ["sui-index"]