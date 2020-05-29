#  Copyright (C) Brandon Waite 2020  - All Rights Reserved
#  Unauthorized copying of this file, via any medium, is strictly prohibited
#  Proprietary
#  Updated by Brandon Waite, May 28 2020

.PHONY: clean install

install:
	go-bindata -o src/data.go -ignore='.*\.(go|rs)$$' src
	go build -o /usr/local/bin/scribe src/*.go

install-tty-debug:
	go-bindata -o src/data.go src
	go build -o /usr/local/bin/scribe -ldflags "-X main.TTYSleep=400ms" src/*.go

clean:
	rm -r ~/.scribe

init:
	go install github.com/jteeuwen/go-bindata
	go install github.com/mattn/go-sqlite3

rust:
	cargo build --release
	cp target/release/scribe /usr/local/bin/scribe
