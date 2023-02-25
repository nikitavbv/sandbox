FROM ubuntu:22.04

RUN mkdir /opt/sandbox
WORKDIR /opt/sandbox

COPY ./libtorch/lib /opt/sandbox/libtorch/lib

COPY target/release/server /opt/sandbox/sandbox

ENTRYPOINT /opt/sandbox/sandbox