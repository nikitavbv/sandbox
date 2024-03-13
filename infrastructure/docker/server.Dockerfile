FROM ubuntu:22.04
WORKDIR /app

COPY target/release/sandbox-server /app/app

ENTRYPOINT [ "/app/app" ]
