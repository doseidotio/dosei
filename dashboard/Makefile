build:
	npm run build
dev:
	npm run dev
run:
	npm run start

publish:
	@if [ -z "$(VERSION)" ]; then \
		echo "Error: VERSION not set"; \
		exit 1; \
	fi
	IMAGE="doseidotio/dashboard:$(VERSION)" && \
	docker build --platform=linux/amd64 -t $$IMAGE . && \
	docker save $$IMAGE | ssh ubuntu@dosei.molinaa.com "docker load"
