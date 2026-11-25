"""Emit one readiness record from drill evidence and native Prometheus tests."""

import argparse
import json
import subprocess
from datetime import datetime, timedelta, timezone
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path, help="Reviewed readiness record in JSON")
    parser.add_argument("rules_test", type=Path, help="Prometheus rule unit test file")
    parser.add_argument("--max-drill-age-days", required=True, type=int)
    args = parser.parse_args()
    if args.max_drill_age_days <= 0:
        parser.error("drill age must be positive")

    record = json.loads(args.evidence.read_text(encoding="utf-8"))
    if record.get("type") != "record" or record.get("kind") != "readiness":
        parser.error("evidence must contain exactly one readiness record")
    data = record["data"]
    drill_time = datetime.fromisoformat(data["recovery"]["observedAt"])
    now = datetime.now(timezone.utc)
    data["recovery"]["fresh"] = (
        now - timedelta(days=args.max_drill_age_days) <= drill_time <= now
    )
    result = subprocess.run(
        ["promtool", "test", "rules", args.rules_test.name],
        cwd=args.rules_test.parent,
        capture_output=True,
        text=True,
        check=False,
    )
    data["paging"]["ruleTestsPassed"] = result.returncode == 0
    print(json.dumps(record))


if __name__ == "__main__":
    main()
