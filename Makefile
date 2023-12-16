#!/usr/bin/make -f

all: fmt check test

fmt:
	cargo fmt --all -- --check
	cargo clippy -- -D warnings

check:
	cargo check --tests

test:
	cargo test
