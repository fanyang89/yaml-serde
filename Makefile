M := .cache/makes
$(shell [ -d $M ] || git clone -q https://github.com/makeplus/makes $M)
include $M/init.mk
include $M/rust.mk
include $M/clean.mk
include $M/shell.mk

MAKES-CLEAN := \
  target \
  Cargo.lock \

CARGO-TARGETS := \
  build \
  check \
  install \
  publish \
  test \
  uninstall \
  update \


$(CARGO-TARGETS): $(CARGO)
	cargo $@ $(opts)
