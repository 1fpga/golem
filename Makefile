NPM := $(shell command -v npm 2> /dev/null)
CROSS := $(shell command -v cross 2> /dev/null)
OPENSSL := $(shell command -v openssl 2> /dev/null)
MISTER_IP := 192.168.1.79

src/frontend/dist/main.js: $(wildcard src/frontend/schemas/**/* src/frontend/migrations/**/* src/frontend/src/**/* src/frontend/src/* src/frontend/types/**/* src/frontend/*.json src/frontend/*.js src/frontend/rollup/*.js)
ifndef NPM
	$(error "No `npm` in PATH, please install Node.js and npm, or pass NPM variable with path to npm binary")
endif
	$(NPM) run -w src/frontend/ build

build-frontend: src/frontend/dist/main.js

.PHONY: target/armv7-unknown-linux-gnueabihf/release/one_fpga
target/armv7-unknown-linux-gnueabihf/release/one_fpga: $(wildcard src/**/*.rs) src/frontend/dist/main.js
ifndef CROSS
	$(error "No `cross` in PATH, please install Node.js and npm, or pass NPM variable with path to npm binary")
endif
	$(CROSS) build \
		--target armv7-unknown-linux-gnueabihf \
		--bin one_fpga \
		--no-default-features \
		--features=platform_de10 \
		--release

build-1fpga: target/armv7-unknown-linux-gnueabihf/release/one_fpga

build: build-frontend build-1fpga

build-and-sign: build
ifndef PUBLIC_KEY_PATH
	$(eval PUBLIC_KEY_PATH = $(shell read -p "Enter path to public key: " key; echo $$key))
endif
	$(OPENSSL) pkeyutl -sign \
		-inkey $(PUBLIC_KEY_PATH) \
		-out target/armv7-unknown-linux-gnueabihf/release/one_fpga.sig \
		-rawin -in target/armv7-unknown-linux-gnueabihf/release/one_fpga

deploy-frontend: build-frontend
	rsync -raH --delete src/frontend/dist/ root@$(MISTER_IP):/root/frontend

new-migration:
	mkdir src/frontend/migrations/1fpga/$(shell date +%Y-%m-%d-%H%M%S)_$(name)
	@echo "-- Add your migration here. Comments will be removed." >> src/frontend/migrations/1fpga/$(shell date +%Y-%m-%d-%H%M%S)_$(name)/up.sql
