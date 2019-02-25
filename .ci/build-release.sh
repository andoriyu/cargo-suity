#!/bin/env bash

set -ex

main() {
    local src=$(pwd) \
          stage=$(mktemp -d) \
          out=$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz

    test -f Cargo.lock || cargo generate-lockfile

    cargo install -f cross

    cross rustc --bin cargo-suity --target $TARGET --release -- -C lto

    cp target/$TARGET/release/hello $stage/

    cd $stage
    tar czf $src/$out *
    cd $src

    mb $src/$out $BUILD_ARTIFACTSTAGINGDIRECTORY

    rm -rf $stage
}

main
