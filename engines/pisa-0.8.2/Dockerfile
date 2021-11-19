FROM ubuntu:20.04 AS builder

ENV DEBIAN_FRONTEND 'noninteractive'
RUN apt-get -y -y -o Dpkg::Options::='--force-confdef' -o Dpkg::Options::='--force-confold' update
RUN apt-get -y install cmake build-essential wget libssl-dev g++

WORKDIR /home/pisa

RUN wget https://github.com/Kitware/CMake/releases/download/v3.21.3/cmake-3.21.3.tar.gz
RUN tar xf cmake-3.21.3.tar.gz
RUN cd cmake-3.21.3 && ./configure && make

ENV PATH /home/pisa/cmake-3.21.3/bin:$PATH

WORKDIR /home/pisa/src

RUN apt-get -y install libtool m4 automake


ENTRYPOINT ["make", "compile-within-container"]

