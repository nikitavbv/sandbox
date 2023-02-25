FROM ubuntu:22.04

RUN mkdir /opt/sandbox
WORKDIR /opt/sandbox

COPY target/release/sandbox /opt/sandbox/sandbox

ENTRYPOINT /opt/sandbox/sandbox