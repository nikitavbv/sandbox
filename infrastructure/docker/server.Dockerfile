FROM frolvlad/alpine-glibc:glibc-2.34
WORKDIR /app

COPY target/release/starhaven /app/app

ENTRYPOINT [ "/app/app" ]