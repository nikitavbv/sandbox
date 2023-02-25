FROM ubuntu:22.04

WORKDIR /app

COPY target/release/sandbox /app/app

ENTRYPOINT /app/app