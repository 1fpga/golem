NPM := $(shell command -v npm 2> /dev/null)
CROSS := $(shell command -v cross 2> /dev/null)

src/golem-frontend/dist/main.js: $(wildcard src/golem-frontend/src/**/*) $(wildcard src/golem-frontend/*.json) $(wildcard src/golem-frontend/*.js)
ifndef NPM
	$(error "No `npm` in PATH, please install Node.js and npm, or pass NPM variable with path to npm binary")
endif
	$(NPM) run -w src/golem-frontend/ build

build-frontend: src/golem-frontend/dist/main.js

target/armv7-unknown-linux-gnueabihf/debug/golem: $(wildcard src/**/*) src/golem-frontend/dist/main.js
ifndef CROSS
	$(error "No `cross` in PATH, please install Node.js and npm, or pass NPM variable with path to npm binary")
endif
	$(CROSS) build --target armv7-unknown-linux-gnueabihf --bin golem --no-default-features --features=platform_de10

build-golem: target/armv7-unknown-linux-gnueabihf/debug/golem

build: build-frontend build-golem
