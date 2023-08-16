FROM frolvlad/alpine-glibc:glibc-2.34
WORKDIR /app

RUN apk add libstdc++

COPY target/release/sandbox-server /app/app

ENTRYPOINT [ "/app/app" ]