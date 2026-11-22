# The whole portal is one image (G1, D33): Toolbox server + OpenFGA + manifest + UI bundles + executors.
# Base pinned by digest for OpenSSF Pinned-Dependencies. Swap to the published
# ghcr.io/<owner>/toolbox@sha256:… digest once the first release ships.
FROM debian:bookworm-slim@sha256:734a72b3d6142a71993b4bc580ca01d8ca687f77a2c422d96a961d3376b179bc
COPY toolbox.yaml /portal/toolbox.yaml
COPY authz/ /portal/authz/
COPY ui/install-approved-app/dist/ /portal/ui/install-approved-app/dist/
COPY executors/github-apps/bin/ /portal/executors/github-apps/bin/
# In-image executors run as a separate unprivileged user with only their declared secrets (Q12).
ENV TOOLBOX_MANIFEST=/portal/toolbox.yaml
# Cloud Run: one service, instance-based billing, min instances >= 1 (plan §6.2, Q13).
CMD ["toolbox", "serve", "--role", "all"]
