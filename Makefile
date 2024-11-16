
run:
	cargo run -- -f data

benchmark:
	cargo build --release --target-dir tmp
	cargo build --target-dir tmp
	hyperfine -N --warmup 3 \
		"tmp/debug/fortune-rs /opt/homebrew/Cellar/fortune/9708/share/games/fortunes" \
		"tmp/release/fortune-rs /opt/homebrew/Cellar/fortune/9708/share/games/fortunes" \
		"fortune -f /opt/homebrew/Cellar/fortune/9708/share/games/fortunes"

clean:
	rm -rf tmp