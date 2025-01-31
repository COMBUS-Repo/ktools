#!/bin/bash

export CC=/usr/local/bin/clang
export CXX=/usr/local/bin/clang++

LOG_BUILD='/tmp/log/ktools-build.log'
ARCH_TARGET=$(uname --m)
BIN_TARGET='/var/ktools'
TAR_TARGET="/var/ktools/tar/Linux_$ARCH_TARGET.tar.gz"
mkdir -p $(dirname $LOG_BUILD)
mkdir -p $(dirname $TAR_TARGET)

./autogen.sh
./configure --enable-o3 --enable-static --prefix=$BIN_TARGET CPPFLAGS=" -D OASIS_FLOAT_TYPE_DOUBLE -D AREAPERIL_TYPE_UNSIGNED_LONG_LONG"
make clean
# make check | tee /tmp/log/ktools-build.log

set +exu
KTOOLS_BUILD_FAILED=$(cat $LOG_BUILD | grep -ci 'Ktools build failed')
# ALL_KTESTS_PASS=$(cat $LOG_BUILD | grep -ci 'All tests passed.')
set -exu

if [ $KTOOLS_BUILD_FAILED -ne 0 ]; then
    echo "Error Detected in Ktools install"
    exit 1
else
    echo "Ktools installed successfully"

    # Create tar
    make install 
    cd $BIN_TARGET/bin
    tar -zcvf $TAR_TARGET ./
    exit 0
fi

