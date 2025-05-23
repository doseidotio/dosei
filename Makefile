export DOCKER_CLI_HINTS=false

build:
	cargo build --release

install i:
	sudo -v  # Prompt for password early and cache credentials
	cargo build -p dosei
	sudo ln -sf "$(PWD)/target/debug/dosei" /usr/local/bin/dosei
	dosei completion bash > /tmp/dosei_completion.bash
	echo "Dosei CLI was installed successfully $(which dosei)"
	echo "Run 'dosei completion bash > /tmp/dosei_completion.bash && source /tmp/dosei_completion.bash' to get dosei bash completion"

dev:
	docker exec -it dosei cargo run --bin doseid

lint:
	docker exec -it dosei cargo fmt
	docker exec -it dosei cargo clippy --release --all-targets --all-features -- -D clippy::all

fix:
	docker exec -it dosei cargo fix --allow-dirty
	docker exec -it dosei cargo clippy --fix --allow-dirty

test:
	docker exec -it dosei cargo test

migrate:
	docker exec -it dosei sh -c "cd doseid && cargo sqlx migrate run"

prepare:
	docker exec -it dosei  sh -c "cd doseid && cargo sqlx prepare"

publish: lint prepare
	VERSION=$$(grep -A 2 "\[workspace.package\]" Cargo.toml | grep "version" | cut -d'"' -f2) && \
	IMAGE="doseidotio/doseid:$$VERSION" && \
	docker rmi doseidotio/doseid:* 2>/dev/null || true && \
	docker build --platform=linux/amd64 -t $$IMAGE . && \
	docker save $$IMAGE | ssh ubuntu@dosei.molinaa.com "docker load && docker image prune -f"

publish.dashboard:
	VERSION=$$(grep -A 2 "\[workspace.package\]" Cargo.toml | grep "version" | cut -d'"' -f2); \
	$(MAKE) -C dashboard publish VERSION=$$VERSION

run:
	docker exec -it dosei bash

exec:
	@if [ -z "$(CMD)" ]; then \
		echo "Error: Please provide a command with CMD="; \
		exit 1; \
	fi
	docker exec -it dosei $(CMD)
