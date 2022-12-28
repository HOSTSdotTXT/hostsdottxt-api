FROM rust:1.66-bullseye AS builder
WORKDIR /src/
RUN cargo init --bin
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
COPY . /src/
RUN touch src/main.rs && cargo build --release


FROM scratch AS bin
COPY --from=builder /src/target/release/hdt-api /hdt-api


FROM debian:bullseye AS deb-builder
ARG VERSION
WORKDIR /root/
COPY pkg/ /root/pkg/
COPY --from=bin /hdt-api pkg/usr/bin/hdt-api
RUN sed -i "s/[{][{] VERSION [}][}]/$(pkg/usr/bin/hdt-api --version)/g" ./pkg/DEBIAN/control
RUN dpkg -b pkg hdt-api_"$(pkg/usr/bin/hdt-api --version)"_amd64.deb


FROM scratch AS deb
ARG VERSION
COPY --from=deb-builder /root/hdt-api_*_amd64.deb /
