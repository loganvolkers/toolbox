"""prompt_check.py <transcript.json> <rendered.md>: does the run's first user message equal its committed prompt?"""
import json
import sys

msgs = json.load(open(sys.argv[1]))["messages"]
sent = next(m["text"] for m in msgs if m["role"] == "user")
want = open(sys.argv[2]).read()
same = sent.strip() == want.strip()
print(json.dumps({"prompt": sys.argv[2], "identical": same}))
sys.exit(0 if same else 1)
