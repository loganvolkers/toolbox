FROM debian:trixie-slim@sha256:8aa5a7bf5a1955bddbe7d74fcdd0b91ab30e74362a8b73af672b0481b35b44a0
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates git \
 && rm -rf /var/lib/apt/lists/*
COPY --chmod=0755 bin/ /usr/local/bin/
COPY --chown=65532:65532 portal/ /portal/
WORKDIR /portal
ENV PORT=8080 SEIKIKA_STATE_DIR=/tmp/seikika SEIKIKA_OPENFGA_BIN=/usr/local/bin/openfga
ENV SEIKIKA_POSTGRES_MAX_CONNECTIONS=4 \
    OPENFGA_DATASTORE_MAX_OPEN_CONNS=4 \
    OPENFGA_DATASTORE_CONN_MAX_IDLE_TIME=1m
USER 65532:65532
EXPOSE 8080
ENTRYPOINT ["seikika"]
CMD ["serve", "/portal"]
