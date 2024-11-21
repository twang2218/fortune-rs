
run:
	cargo run --bin fortune -- tests/data

build:
	cargo build --release
	cargo build

test:
	RUST_BACKTRACE=all cargo test

benchmark:
	$(eval cookies := /opt/homebrew/Cellar/fortune/9708/share/games/fortunes)
	cargo build --release --target-dir tmp
	cargo build --target-dir tmp
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
clean:
	rm -rf tmp
