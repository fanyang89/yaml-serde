M := .cache/makes
$(shell [ -d $M ] || git clone -q https://github.com/makeplus/makes $M)
include $M/init.mk
include $M/rust.mk
include $M/yamlscript.mk
include $M/clean.mk
include $M/shell.mk

MAKES-CLEAN := \
  target \
  Cargo.lock \

CARGO-TARGETS := \
  build \
  check \
  install \
  test \
  uninstall \
  update \

SECRETS-FILE := $(HOME)/.yaml-serde-secrets.yaml


$(CARGO-TARGETS): $(CARGO)
	cargo $@ $(opts)

publish: $(CARGO) $(YS)
ifeq (,$(wildcard $(SECRETS-FILE)))
	@echo 'ERROR: $(SECRETS-FILE) not found' >&2
	exit 1
else
	CARGO_TOKEN=$$(ys -e '.crates.token:say' $(SECRETS-FILE)) \
	  $(CARGO) publish --token "$$CARGO_TOKEN"
endif
