# Change to build a different example, like otp_async
ARG EXAMPLE=otp

FROM rust:alpine as base
ARG EXAMPLE

RUN apk --no-cache add \
    git \
    gcc \
    g++ \
    openssl \
    openssl-dev \
    pkgconfig

COPY . /src

WORKDIR /src

ENV RUSTFLAGS="-C target-feature=-crt-static"
RUN cargo build \
    --release \
    --example "${EXAMPLE}"

FROM alpine:3
ARG EXAMPLE
RUN apk --no-cache add \
    libgcc \
    pcsc-lite-dev

COPY --from=base "/src/target/release/examples/${EXAMPLE}" /usr/local/bin/otp

ENV RUST_BACKTRACE=1
ENTRYPOINT [ "/usr/local/bin/otp" ]
