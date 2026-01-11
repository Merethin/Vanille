
FROM rust:1.90-alpine AS build

WORKDIR /usr/src/vanille
RUN cargo init --bin .

RUN apk add libressl-dev musl-dev

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/vanille*
RUN cargo build --release

FROM rust:1.90-alpine

WORKDIR /

COPY --from=build /usr/src/vanille/target/release/vanille /usr/local/bin/vanille

CMD ["vanille"]
