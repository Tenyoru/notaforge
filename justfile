CONFIG_PATH := "config.toml"

build:
	cargo build

clean:
	cargo clean

run term="aback":
	cargo run -- --config {{CONFIG_PATH}} --term {{term}}

test:
	cargo test
