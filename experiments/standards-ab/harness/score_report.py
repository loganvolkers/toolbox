"""Compare an arm's Report JSON (list of {file, count|lines}) with the ground truth. Usage: score_report.py <truth.json> <report.json>"""
import json, sys
truth = json.load(open(sys.argv[1])); raw = json.load(open(sys.argv[2]))
if isinstance(raw, dict): raw = raw.get("violations") or raw.get("files") or list(raw.values())[0]
got = {r["file"].lstrip("./"): r.get("count", r.get("lines")) for r in raw}
tp = set(got) & set(truth); fp = set(got) - set(truth); fn = set(truth) - set(got)
exact = sum(1 for f in tp if got[f] == truth[f])
print(json.dumps({"truth_files": len(truth), "report_files": len(got), "precision": round(len(tp)/max(len(got),1), 3),
                  "recall": round(len(tp)/max(len(truth),1), 3), "exact_values": f"{exact}/{len(tp)}", "fp": sorted(fp)[:5], "fn": sorted(fn)[:5]}))
