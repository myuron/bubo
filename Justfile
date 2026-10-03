default:
  @just --list

fmt:
  cargo fmt

lint:
  cargo clippy

test:
  cargo test

build:
  cargo build
