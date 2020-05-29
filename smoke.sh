#!/bin/sh

# Copyright (C) Brandon Waite 2020  - All Rights Reserved
# Unauthorized copying of this file, via any medium, is strictly prohibited
# Proprietary
# Updated by Brandon Waite, May 28 2020

mkdir -p .tests && rm .tests/*

run() {
    name=$1; shift

    zig run --release-safe src/main.zig -- $@ >> .tests/$name.zig.out 2>> .tests/$name.zig.err
    cargo run --release --quiet -- $@ >> .tests/$name.rust.out 2>> .tests/$name.rust.err
    go build -mod=vendor -o target/scribe src/main.go && \
        target/scribe $@ >> .tests/$name.go.out 2>> .tests/$name.go.err
}

compare() {
    name=$1

    if diff .tests/$name.go.out .tests/$name.zig.out; then
        if diff .tests/$name.go.out .tests/$name.rust.out; then
            echo "Passed on test: '$name'"
        else
            echo "Failure on test: '$name'"
        fi
    else
        echo "Failure on test: '$name'"
    fi
}

do_test() {
    name=$1; shift
    echo "Running $name"

    run $name $@
    compare $name
}

do_test init init
do_test record record a b c d e

echo "Zig impl stats"
ls -al ./zig-cache/bin/scribe

echo "Rust impl stats"
ls -al ./target/release/scribe

echo "Go impl stats"
ls -al ./target/scribe
