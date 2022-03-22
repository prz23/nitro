#!/usr/bin/env bash
set -e

export CC=musl-gcc
export CXX=g++

THISDIR="$(dirname $(readlink -f $0))"
INSTALLDIR="/usr/local/x86_64-linux-musl"
DEPSDIR="$THISDIR/deps"
TARGET_SO="$DEPSDIR/openssl/libcrypto.so"
SGX_VER="2.11"

mkdir -p $DEPSDIR || exit 1

# Download OpenSSL 1.1.1
OPENSSLDIR="${DEPSDIR}/openssl"
if [ ! -d "$OPENSSLDIR" ] ; then
    echo "Downloading openssl ..."
    cd "$DEPSDIR" && \
    wget https://github.com/openssl/openssl/archive/OpenSSL_1_1_1.tar.gz && \
    tar -xvzf OpenSSL_1_1_1.tar.gz && \
    mv openssl-OpenSSL_1_1_1 openssl && \
    echo "Download openssl successfully" || exit 1
else
    echo "The openssl code is already existent"
fi

# Build openssl
if [ ! -f "$TARGET_SO" ] ; then
    echo "Building openssl ..."
    cd "$OPENSSLDIR" && \
    CC=musl-gcc ./config \
      --prefix=$INSTALLDIR \
      --openssldir=/usr/local/ssl \
      --with-rand-seed=rdcpu \
      no-zlib no-async no-tests && \
    make -j${nproc} && make install && \
    echo "Build openssl successfully" || exit 1
else
    echo "The openssl library is aleady existent"
fi

echo "deps is ready"
