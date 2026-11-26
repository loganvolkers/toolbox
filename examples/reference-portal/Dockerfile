# The whole portal is one image (G1, D33): Seikika server + OpenFGA + manifest + UI bundles + executors.
# Base pinned by digest for OpenSSF Pinned-Dependencies. Swap to the published
# ghcr.io/<owner>/seikika@sha256:… digest once the first release ships.
FROM debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc
COPY seikika.yaml /portal/seikika.yaml
COPY authz/ /portal/authz/
COPY ui/install-approved-app/dist/ /portal/ui/install-approved-app/dist/
COPY executors/github-apps/bin/ /portal/executors/github-apps/bin/
# In-image executors run as a separate unprivileged user with only their declared secrets (Q12).
ENV SEIKIKA_MANIFEST=/portal/seikika.yaml
# Cloud Run: one service, instance-based billing, min instances >= 1 (plan §6.2, Q13).
CMD ["seikika", "serve", "--role", "all"]
