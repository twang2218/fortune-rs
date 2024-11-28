COOKIES_DIR := /opt/homebrew/Cellar/fortune/9708/share/games/fortunes
TARGETS_LINUX := \
	x86_64-unknown-linux-gnu \
	aarch64-unknown-linux-gnu
TARGETS_MACOS := \
	x86_64-apple-darwin \
	aarch64-apple-darwin
TARGETS_WINDOWS := \
	x86_64-pc-windows-gnu

TARGETS := $(TARGETS_MACOS) $(TARGETS_LINUX) $(TARGETS_WINDOWS)

.PHONY: run build pre-build test benchmark clean
run:
	cargo run --bin fortune -- tests/data

test:
	RUST_BACKTRACE=all cargo test

build: $(TARGETS)

$(TARGETS):
	cross build --release --target $@

pre-build:
	cargo install cross --git https://github.com/cross-rs/cross
	rustup target add $(TARGETS)

benchmark:
	$(eval cookies := $(COOKIES_DIR))
	cargo build --release --target-dir tmp
	$(eval args := -f)
	hyperfine -N --warmup 3 \
		"tmp/release/fortune $(args) $(cookies)" \
		"fortune $(args) $(cookies)"
	$(eval args := )
	hyperfine -N --warmup 3 \
		"tmp/release/fortune $(args) $(cookies)" \
		"fortune $(args) $(cookies)"
	$(eval args := -i -m lucky)
	hyperfine -i -N --warmup 3 \
		"tmp/release/fortune $(args) $(cookies)" \
		"fortune $(args) $(cookies)"
	rm -rf tmp

coverage:
	cargo tarpaulin --out Html --output-dir output

clean:
	rm -rf tmp
	cargo clean
