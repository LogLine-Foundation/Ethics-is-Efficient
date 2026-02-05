
SHELL := /bin/bash
DIST := dist
MUSL_TARGET := x86_64-unknown-linux-musl
WASM_TARGET := wasm32-unknown-unknown

.PHONY: all build-native build-wasm dist clean

all: dist build-native build-wasm

dist:
	mkdir -p $(DIST)

build-native: dist
	# Resolver binary (musl static)
	rustup target add $(MUSL_TARGET) || true
	cargo build --release --target $(MUSL_TARGET) -p resolver
	cp target/$(MUSL_TARGET)/release/resolver $(DIST)/resolver

	# Offline verifier (musl static, if present)
	@if cargo metadata --no-deps --format-version=1 | jq -r '.packages[].name' | grep -q '^tdln-verify$$'; then \
		cargo build --release --target $(MUSL_TARGET) -p tdln-verify ; \
		cp target/$(MUSL_TARGET)/release/tdln-verify $(DIST)/tdln-verify ; \
	fi

build-wasm: dist
	rustup target add $(WASM_TARGET) || true
	cargo build --release --target $(WASM_TARGET) -p guest-ack
	cargo build --release --target $(WASM_TARGET) -p guest-ask
	cp target/$(WASM_TARGET)/release/guest_ack.wasm $(DIST)/guest-ack.wasm
	cp target/$(WASM_TARGET)/release/guest_ask.wasm $(DIST)/guest-ask.wasm

clean:
	cargo clean
	rm -rf $(DIST)


IMAGE ?= resolver:local

.PHONY: docker-build docker-run docker-distroless

docker-build: build-native
	# Build scratch image (binary must be musl static)
	docker build -f container/Dockerfile.scratch -t $(IMAGE) .

docker-distroless: build-native
	# Build distroless image alternative
	docker build -f container/Dockerfile.distroless -t $(IMAGE) .

docker-run:
	docker run --rm -p 8080:8080 $(IMAGE)


AARCH64_TARGET := aarch64-unknown-linux-musl

.PHONY: build-native-arm build-native-all

build-native-arm: dist
	rustup target add $(AARCH64_TARGET) || true
	cargo build --release --target $(AARCH64_TARGET) -p resolver
	cp target/$(AARCH64_TARGET)/release/resolver dist/resolver-arm64

build-native-all: build-native build-native-arm


.PHONY: docker-buildx

# Buildx multi-arch (requires dist/resolver-amd64 and dist/resolver-arm64 present)
docker-buildx: build-native-all
	@if ! docker buildx ls >/dev/null 2>&1; then echo ">> Creating buildx builder"; docker buildx create --use; fi
	docker buildx build     	  --platform linux/amd64,linux/arm64     	  -f container/Dockerfile.scratch     	  -t $(IMAGE)     	  --push     	  .
