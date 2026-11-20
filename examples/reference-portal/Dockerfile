# The whole portal is one image (G1, D33): Toolbox server + OpenFGA + manifest + UI bundles + executors.
# The base image name is illustrative until the first release.
FROM ghcr.io/toolbox-dev/toolbox:0.1
COPY toolbox.yaml /portal/toolbox.yaml
COPY authz/ /portal/authz/
COPY ui/install-approved-app/dist/ /portal/ui/install-approved-app/dist/
COPY executors/github-apps/bin/ /portal/executors/github-apps/bin/
# In-image executors run as a separate unprivileged user with only their declared secrets (Q12).
ENV TOOLBOX_MANIFEST=/portal/toolbox.yaml
# Cloud Run: one service, instance-based billing, min instances >= 1 (plan §6.2, Q13).
CMD ["toolbox", "serve", "--role", "all"]
