# Acme portal image: toolbox, its in-image connectors, the SCIM connector, OpenFGA, and this portal.
# Binaries are copied in by the harness (cargo already built them). The base matches the glibc
# those host-built binaries need. OpenFGA's digest matches image/Dockerfile.
FROM ubuntu:24.04@sha256:9c7f200c07e3d4a03e7249d2c16bf66a51120fb876aa2242cf31451412543289
ARG OPENFGA_VERSION=1.22.1
ARG OPENFGA_SHA256_amd64=80d2d4b582762c02dc86b1246f7f6f54d6854259727b36e5b5c43ba90792814a
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates curl \
 && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL -o /tmp/openfga.tgz "https://github.com/openfga/openfga/releases/download/v${OPENFGA_VERSION}/openfga_${OPENFGA_VERSION}_linux_amd64.tar.gz" \
 && echo "${OPENFGA_SHA256_amd64}  /tmp/openfga.tgz" | sha256sum -c - \
 && tar -xzf /tmp/openfga.tgz -C /usr/local/bin openfga \
 && rm /tmp/openfga.tgz
COPY bin/ /usr/local/bin/
COPY portal/ /portal/
COPY entrypoint.sh /entrypoint.sh
RUN chmod 755 /entrypoint.sh /usr/local/bin/toolbox /usr/local/bin/toolbox-connector-github /usr/local/bin/toolbox-connector-git-files /usr/local/bin/toolbox-connector-scim
ENV TOOLBOX_OPENFGA_BIN=/usr/local/bin/openfga
ENV PORT=8080
USER 65532:65532
EXPOSE 8080 9100
ENTRYPOINT ["/entrypoint.sh"]
