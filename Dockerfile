# Builder
FROM rust:1.83-slim as builder

ARG SOURCE_DIR

WORKDIR /usr/src/${SOURCE_DIR}
COPY ./${SOURCE_DIR} .
COPY ./shared ../shared

RUN cargo install --path .

# Runner
FROM debian:stable-slim

ARG SOURCE_DIR

COPY --from=builder /usr/local/cargo/bin/${SOURCE_DIR} /usr/local/bin/${SOURCE_DIR}

ENV SOURCE_DIR=${SOURCE_DIR}

EXPOSE 8080

CMD "${SOURCE_DIR}"
