"""Ground truth for the standards A/B. Usage: truth.py s1|s1b|s2|s2b|s3 <repo dir> -> JSON {file: n}."""
import json, os, re, sys
kind, root = sys.argv[1], sys.argv[2]
def is_test_py(p):
    b = os.path.basename(p)
    return ('/tests/' in '/'+p or '/test/' in '/'+p or b.startswith('test_') or b.endswith('_test.py') or b == 'conftest.py')
out = {}
if kind in ('s1', 's1b'):
    models = ['Person', 'PersonDistinctId', 'CohortPeople'] + (['Group', 'GroupTypeMapping'] if kind == 's1b' else [])
    rx = re.compile(r'\b(' + '|'.join(models) + r')\.objects\b')
    exempt = {'posthog/models/person/util.py', 'posthog/models/group_type_mapping.py'} | ({'posthog/models/person/person.py'} if kind == 's1b' else set())
    for d, ds, fs in os.walk(root):
        ds[:] = [x for x in ds if x not in ('.git', 'node_modules', '.venv')]
        for f in fs:
            if not f.endswith('.py'): continue
            p = os.path.relpath(os.path.join(d, f), root)
            if is_test_py(p) or p in exempt: continue
            n = len(rx.findall(open(os.path.join(d, f), errors='ignore').read()))
            if n: out[p] = n
else:
    ext, base, limit = {'s2': ('.rs', 'codex-rs', 800), 's2b': ('.rs', 'codex-rs', 700), 's3': ('.go', '.', 1000)}[kind]
    for d, ds, fs in os.walk(os.path.join(root, base)):
        # S2 follows Codex's rule (tests/ directories are tests); S3's prompt exempts only _test.go
        ds[:] = [x for x in ds if x not in ('.git', 'node_modules', 'target') + (('tests',) if kind != 's3' else ())]
        for f in fs:
            if not f.endswith(ext) or f.endswith('_test.go') or f == 'tests.rs' or f.endswith('_tests.rs'): continue
            full = os.path.join(d, f); p = os.path.relpath(full, root)
            n = sum(1 for _ in open(full, errors='ignore'))
            if n > limit: out[p] = n
print(json.dumps(out, sort_keys=True))
