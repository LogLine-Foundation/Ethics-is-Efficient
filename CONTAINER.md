
# Container Guide

Two minimal options are provided:
- `container/Dockerfile.scratch`: ultra-minimal (FROM scratch), needs MUSL static binary.
- `container/Dockerfile.distroless`: safer defaults (Distroless nonroot).

## Build & Run (scratch)
```bash
make build-native                   # produces dist/resolver
make docker-build IMAGE=resolver:local
make docker-run   IMAGE=resolver:local
# open http://localhost:8080/healthz (if implemented) or run flows:
curl -s localhost:8080/v2/run -H 'content-type: application/json' -d '{"realm":"trust","inputs":{"policy":{"kind":"noop"}}}'
```

## Build & Run (distroless)
```bash
make docker-distroless IMAGE=resolver:distroless
make docker-run       IMAGE=resolver:distroless
```

## Keys & Base URL
The resolver will generate signing keys in `./keys/` on first run if none exist.
For production bind-mount or bake keys:
```bash
docker run --rm -p 8080:8080 -v $(pwd)/keys:/keys -e TDLN_KEYS_DIR=/keys resolver:local
```

You can also set `BASE_URL` to control the canonical links rendered on HTML pages:
```bash
docker run --rm -p 8080:8080 -e BASE_URL="https://cert.example.com" resolver:local
```

## Multi-arch (optional)
If you need multi-arch images, build musl binaries for each target and use buildx:
```bash
docker buildx build --platform linux/amd64,linux/arm64 -f container/Dockerfile.scratch -t ghcr.io/yourorg/resolver:latest --push .
```


## Multi-arch images (amd64 + arm64)
Build both static binaries, then build & push a multi-arch image with buildx:
```bash
make build-native-all
# produce dist/resolver-amd64 and dist/resolver-arm64
docker buildx build       --platform linux/amd64,linux/arm64       -f container/Dockerfile.scratch       -t ghcr.io/yourorg/resolver:latest       --push .
```

Locally (single-arch), you can still do:
```bash
# amd64 example
cp dist/resolver dist/resolver-amd64
docker build -f container/Dockerfile.scratch --build-arg TARGETARCH=amd64 -t resolver:local .
```
