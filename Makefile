
run:
	cargo run --bin fortune -- -f data

build:
	cargo build --release
	cargo build

benchmark:
	cargo build --release --target-dir tmp
	cargo build --target-dir tmp
	hyperfine -N --warmup 3 \
		"tmp/debug/fortune /opt/homebrew/Cellar/fortune/9708/share/games/fortunes" \
		"tmp/release/fortune /opt/homebrew/Cellar/fortune/9708/share/games/fortunes" \
		"fortune -f /opt/homebrew/Cellar/fortune/9708/share/games/fortunes"

clean:
	rm -rf tmp