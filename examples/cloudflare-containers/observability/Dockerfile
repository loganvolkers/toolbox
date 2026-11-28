FROM grafana/otel-lgtm:0.41.2@sha256:79dbbe04d810bfe93fd0310610a1643ba4ed420e7432143a810e71e9cba2541e

ENV GF_AUTH_ANONYMOUS_ENABLED=true \
    GF_AUTH_ANONYMOUS_ORG_ROLE=Viewer \
    GF_AUTH_DISABLE_LOGIN_FORM=true \
    GF_AUTH_BASIC_ENABLED=false \
    GF_SECURITY_DISABLE_INITIAL_ADMIN_CREATION=true \
    GF_USERS_ALLOW_SIGN_UP=false \
    GF_ANALYTICS_REPORTING_ENABLED=false \
    GF_ANALYTICS_CHECK_FOR_UPDATES=false \
    GF_PLUGINS_PREINSTALL_DISABLED=true \
    ENABLE_LOGS_OTELCOL=true \
    PROMETHEUS_EXTRA_ARGS=--storage.tsdb.retention.time=48h

COPY collector.yaml /otel-lgtm/otelcol-config.yaml
COPY dashboard.yml /otel-lgtm/grafana/conf/provisioning/dashboards/grafana-dashboards.yaml
COPY dashboard.json /otel-lgtm/seikika-runtime/dashboard.json
