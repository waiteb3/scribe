#  Copyright (C) Brandon Waite 2020  - All Rights Reserved
#  Unauthorized copying of this file, via any medium, is strictly prohibited
#  Proprietary
#  Updated by Brandon Waite, May 28 2020

.PHONY: build

build:
	cargo build --release

install: build
	cp target/release/scribe /usr/local/bin/scribe
	scribe version

uninstall:
	rm -r ~/.scribe
	rm $(which scribe)
