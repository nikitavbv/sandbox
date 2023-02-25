FROM ubuntu:22.04

RUN mkdir /opt/sandbox
WORKDIR /opt/sandbox

COPY ./linux-gpu-env.sh /opt/sandbox/linux-gpu-env.sh
RUN /opt/sandbox/linux-gpu-env.sh

RUN apt update && apt install libgomp1 wget unzip

ENV LIBTORCH=/opt/sandbox/libtorch
ENV LD_LIBRARY_PATH=/opt/sandbox/libtorch/lib

COPY target/release/server /opt/sandbox/sandbox

ENTRYPOINT /opt/sandbox/sandbox