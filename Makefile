.PHONY: build install

build:
	cargo build

install:
	./scripts/install-macos.sh
