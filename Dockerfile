
FROM rust:1.90-alpine AS build

WORKDIR /usr/src/vanille
RUN cargo init --bin .

RUN mkdir -p ./caramel
RUN cargo init --bin caramel

RUN mkdir -p ./models
RUN cargo init --bin models

RUN mkdir -p ./graph
RUN cargo init --bin graph

RUN apk add libressl-dev musl-dev py3-matplotlib

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./caramel/Cargo.lock ./caramel/Cargo.lock
COPY ./caramel/Cargo.toml ./caramel/Cargo.toml
COPY ./models/Cargo.lock ./models/Cargo.lock
COPY ./models/Cargo.toml ./models/Cargo.toml
COPY ./graph/Cargo.lock ./graph/Cargo.lock
COPY ./graph/Cargo.toml ./graph/Cargo.toml

RUN cargo build --release
RUN rm src/*.rs
RUN rm caramel/src/*.rs
RUN rm models/src/*.rs
RUN rm graph/src/*.rs

COPY ./src ./src
COPY ./caramel/src ./caramel/src
COPY ./models/src ./models/src
COPY ./graph/src ./graph/src

RUN rm ./target/release/deps/vanille*
RUN cargo build --release

FROM rust:1.90-alpine

WORKDIR /

STOPSIGNAL SIGINT

COPY --from=build /usr/src/vanille/target/release/vanille /usr/local/bin/vanille

CMD ["vanille"]
