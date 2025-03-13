build:
	cargo build --release

cli.build:
	cargo build -p dosei

dev:
	cargo run --bin doseid

lint:
	cargo fmt
	cargo clippy --release --all-targets --all-features -- -D clippy::all

migrate:
	cd doseid && cargo sqlx migrate run

prepare:
	cd doseid && cargo sqlx prepare
	cd ..
	cd proxy && cargo sqlx prepare
