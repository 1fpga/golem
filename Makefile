NPM := $(shell command -v npm 2> /dev/null)
CROSS := $(shell command -v cross 2> /dev/null)
MISTER_IP := 192.168.1.79

src/golem-frontend/dist/main.js: $(wildcard src/golem-frontend/schemas/**/* src/golem-frontend/src/* src/golem-frontend/src/**/* src/golem-frontend/types/**/* src/golem-frontend/*.json wildcard src/golem-frontend/*.js wildcard src/golem-frontend/rollup/*.js)
ifndef NPM
	$(error "No `npm` in PATH, please install Node.js and npm, or pass NPM variable with path to npm binary")
endif
	$(NPM) run -w src/golem-frontend/ build

build-frontend: src/golem-frontend/dist/main.js

target/armv7-unknown-linux-gnueabihf/debug/golem: $(wildcard src/**/*) src/golem-frontend/dist/main.js
ifndef CROSS
	$(error "No `cross` in PATH, please install Node.js and npm, or pass NPM variable with path to npm binary")
endif
	$(CROSS) build \
		--target armv7-unknown-linux-gnueabihf \
		--bin golem \
		--no-default-features \
		--features=platform_de10 \
		--release

build-golem: target/armv7-unknown-linux-gnueabihf/debug/golem

build: build-frontend build-golem

deploy-frontend: build-frontend
	rsync -raH --delete src/golem-frontend/dist/ root@$(MISTER_IP):/root/frontend

new-migration:
	mkdir src/golem-frontend/migrations/1fpga/$(shell date +%Y-%m-%d-%H%M%S)_$(name)
	@echo "-- Add your migration here. Comments will be removed." >> src/golem-frontend/migrations/1fpga/$(shell date +%Y-%m-%d-%H%M%S)_$(name)/up.sql
