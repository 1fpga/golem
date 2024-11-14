NPM := $(shell command -v npm 2> /dev/null)
CROSS := $(shell command -v cross 2> /dev/null)
OPENSSL := $(shell command -v openssl 2> /dev/null)
MISTER_IP := 192.168.1.79

src/golem-frontend/dist/main.js: $(wildcard src/golem-frontend/schemas/**/* src/golem-frontend/migrations/**/* src/golem-frontend/src/**/* src/golem-frontend/types/**/* src/golem-frontend/*.json src/golem-frontend/*.js src/golem-frontend/rollup/*.js)
ifndef NPM
	$(error "No `npm` in PATH, please install Node.js and npm, or pass NPM variable with path to npm binary")
endif
	$(NPM) run -w src/golem-frontend/ build

build-frontend: src/golem-frontend/dist/main.js

.PHONY: target/armv7-unknown-linux-gnueabihf/release/golem
target/armv7-unknown-linux-gnueabihf/release/golem: $(wildcard src/**/*.rs) src/golem-frontend/dist/main.js
ifndef CROSS
	$(error "No `cross` in PATH, please install Node.js and npm, or pass NPM variable with path to npm binary")
endif
	$(CROSS) build \
		--target armv7-unknown-linux-gnueabihf \
		--bin golem \
		--no-default-features \
		--features=platform_de10 \
		--release

build-golem: target/armv7-unknown-linux-gnueabihf/release/golem

build: build-frontend build-golem

build-and-sign: build
ifndef PUBLIC_KEY_PATH
	PUBLIC_KEY_PATH ?= $(shell read -p "Enter path to public key: " key; echo $$key)
endif
	$(OPENSSL) pkeyutl -sign \
		-inkey $(PUBLIC_KEY_PATH) \
		-out target/armv7-unknown-linux-gnueabihf/release/signature.bin \
		-rawin -in target/armv7-unknown-linux-gnueabihf/release/golem

deploy-frontend: build-frontend
	rsync -raH --delete src/golem-frontend/dist/ root@$(MISTER_IP):/root/frontend

new-migration:
	mkdir src/golem-frontend/migrations/1fpga/$(shell date +%Y-%m-%d-%H%M%S)_$(name)
	@echo "-- Add your migration here. Comments will be removed." >> src/golem-frontend/migrations/1fpga/$(shell date +%Y-%m-%d-%H%M%S)_$(name)/up.sql
