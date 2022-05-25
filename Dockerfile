FROM rust:1.61-bullseye AS builder
WORKDIR /src/
COPY . /src/
RUN cargo build --release


FROM scratch AS bin
COPY --from=builder /src/target/release/fdns-api /fdns-api


FROM debian:buster AS deb-builder
ARG VERSION
WORKDIR /root/
COPY pkg/ /root/pkg/
COPY --from=bin /fdns-api pkg/usr/bin/fdns-api
RUN sed -i "s/[{][{] VERSION [}][}]/$(pkg/usr/bin/fdns-api --version)/g" ./pkg/DEBIAN/control
RUN dpkg -b pkg fdns-api_"$(pkg/usr/bin/fdns-api --version)"_amd64.deb


FROM scratch AS deb
ARG VERSION
COPY --from=deb-builder /root/fdns-api_*_amd64.deb /
