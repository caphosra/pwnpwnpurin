FROM ubuntu:20.04

ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=Etc/UTC

RUN \
    apt update; \
    apt install -y \
        gcc make gdb \
        texinfo gawk bison sed \
        python3-dev python3-pip python-is-python3 \
        wget; \
    pip install pexpect;
