"""Score Collector file exports with data from an actual Seikika request."""

import json
import re
import sys
from argparse import ArgumentParser
from pathlib import Path

PROBED_PUBLIC_PATHS = {"/healthz", "/readyz"}


def attr(record, name, value_type="stringValue"):
    return next(
        (
            entry["value"].get(value_type)
            for entry in record.get("attributes", [])
            if entry["key"] == name
        ),
        None,
    )


def records(directory, signal, resource_key, scope_key, record_key, public):
    service = "seikika-demo" if public else "seikika"
    with (directory / f"{signal}.json").open() as stream:
        for line in stream:
            for batch in json.loads(line)[resource_key]:
                attributes = batch["resource"]["attributes"]
                if attr(batch["resource"], "service.name") != service:
                    raise ValueError(f"Unexpected service name in {signal}")
                if public and len(attributes) != 1:
                    raise ValueError(f"Unexpected public resource attributes in {signal}")
                for scope in batch[scope_key]:
                    yield from scope[record_key]


def check_exports(directory, public):
    spans = list(records(directory, "traces", "resourceSpans", "scopeSpans", "spans", public))
    logs = list(records(directory, "logs", "resourceLogs", "scopeLogs", "logRecords", public))
    if public:
        for span in spans:
            if attr(span, "url.path") not in PROBED_PUBLIC_PATHS or span.get("events"):
                raise ValueError("Public traces contain a private path or span event")
            if {item["key"] for item in span["attributes"]} - {
                "http.request.method", "url.path", "http.response.status_code"
            }:
                raise ValueError("Public traces contain an unexpected span attribute")
        for log in logs:
            if (
                attr(log, "url.path") not in PROBED_PUBLIC_PATHS
                or log["body"].get("stringValue") != "http request completed"
            ):
                raise ValueError("Public logs contain a private path or unexpected body")
            if {item["key"] for item in log["attributes"]} - {
                "http.request.method", "url.path", "status"
            }:
                raise ValueError("Public logs contain an unexpected log attribute")
    request_spans = {
        (span["traceId"], span["spanId"])
        for span in spans
        if span["name"] == "http.server.request"
        and attr(span, "url.path") == "/healthz"
        and attr(span, "http.request.method") == "GET"
        and str(attr(span, "http.response.status_code", "intValue")) == "200"
        and re.fullmatch(r"[a-f0-9]{32}", span["traceId"])
        and re.fullmatch(r"[a-f0-9]{16}", span["spanId"])
    }
    request_logs = {
        (log["traceId"], log["spanId"])
        for log in logs
        if log.get("body", {}).get("stringValue") == "http request completed"
        and attr(log, "url.path") == "/healthz"
        and attr(log, "http.request.method") == "GET"
        and attr(log, "status") == "200"
    }
    required_metrics = {
        "seikika_snapshot_lag_seconds": "gauge",
        "seikika_ingest_records_total": "sum",
        "seikika_ingest_batches_total": "sum",
    }
    exported_metrics = set()
    for metric in records(directory, "metrics", "resourceMetrics", "scopeMetrics", "metrics", public):
        name = metric["name"]
        if public and name not in required_metrics:
            raise ValueError(f"Unexpected public metric {name}")
        points = metric.get(required_metrics.get(name), {}).get("dataPoints", [])
        if name == "seikika_snapshot_lag_seconds":
            points = [point for point in points if point.get("asDouble", 0) > 0]
        if points:
            exported_metrics.add(name)
    missing = required_metrics.keys() - exported_metrics
    if missing:
        raise ValueError(f"Missing Seikika OTLP metric points: {', '.join(sorted(missing))}")
    matching = request_spans & request_logs
    if not matching:
        raise ValueError("No shared OTLP trace and span IDs for GET /healthz (200)")
    with (directory / "server.log").open() as stream:
        local_events = [record.get("fields", {}) for record in map(json.loads, stream)]
    if public and not any(event.get("url.path") == "/v1/snapshot" for event in local_events):
        raise ValueError("The test did not exercise the private snapshot route")
    local_trace_ids = {event.get("trace_id") for event in local_events}
    if not any(trace_id in local_trace_ids for trace_id, _ in matching):
        raise ValueError("No matching trace ID in the portal's JSON stderr")


if __name__ == "__main__":
    parser = ArgumentParser(description=__doc__)
    parser.add_argument("directory", type=Path)
    parser.add_argument("--public", action="store_true")
    args = parser.parse_args()
    try:
        check_exports(args.directory, args.public)
    except (OSError, ValueError, KeyError) as error:
        sys.exit(str(error))
