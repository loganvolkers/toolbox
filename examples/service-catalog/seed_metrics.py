"""Seed the local Prometheus OTLP receiver with the demo and an unrelated service."""

import json
from time import time_ns
from urllib.request import Request, urlopen

import sys


def resource(name, value):
    return {
        "resource": {"attributes": [{"key": "service.name", "value": {"stringValue": name}}]},
        "scopeMetrics": [{"metrics": [{
            "name": "seikika_snapshot_lag_seconds",
            "gauge": {"dataPoints": [{"timeUnixNano": str(time_ns()), "asDouble": value}]},
        }]}],
    }


payload = {"resourceMetrics": [resource("seikika-demo", 12.5), resource("other", 99)]}
request = Request(
    f"{sys.argv[1]}/api/v1/otlp/v1/metrics",
    data=json.dumps(payload).encode(),
    headers={"Content-Type": "application/json"},
)
with urlopen(request, timeout=10) as response:
    assert response.status == 200
