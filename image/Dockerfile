# The Toolbox server image (charter G1, D90): the released Linux binaries of toolbox and its builtin
# connectors, plus OpenFGA, which `toolbox serve` supervises. Built by .github/workflows/image.yml
# from a release's own archives (nothing is compiled here). Operators build FROM it and add their
# portal:  FROM ghcr.io/<owner>/toolbox:<version>  then  COPY portal/ /portal/
# Digest for debian:bookworm-slim (multi-arch index); OpenSSF Pinned-Dependencies.
FROM debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc
ARG TARGETARCH
ARG OPENFGA_VERSION=1.22.1
ARG OPENFGA_SHA256_amd64=80d2d4b582762c02dc86b1246f7f6f54d6854259727b36e5b5c43ba90792814a
ARG OPENFGA_SHA256_arm64=57678dc8697a224cc867081993d3a3676e005ce19d787463c913c73aed75cf5c
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl git \
 && rm -rf /var/lib/apt/lists/*
RUN set -eu; \
    case "$TARGETARCH" in amd64) sha="$OPENFGA_SHA256_amd64" ;; arm64) sha="$OPENFGA_SHA256_arm64" ;; *) exit 1 ;; esac; \
    curl -fsSL -o /tmp/openfga.tgz "https://github.com/openfga/openfga/releases/download/v${OPENFGA_VERSION}/openfga_${OPENFGA_VERSION}_linux_${TARGETARCH}.tar.gz"; \
    echo "$sha  /tmp/openfga.tgz" | sha256sum -c -; \
    tar -xzf /tmp/openfga.tgz -C /usr/local/bin openfga; \
    rm /tmp/openfga.tgz
COPY bin/${TARGETARCH}/ /usr/local/bin/
ENV TOOLBOX_OPENFGA_BIN=/usr/local/bin/openfga
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["toolbox"]
CMD ["serve", "/portal", "--listen", "0.0.0.0:8080"]
