#!/usr/bin/env python3
"""README §6, by actions-ab's tally.py over this benchmark's two arms.

    tally.py <results/r<n>>
"""
import importlib.util
import os
import sys

spec = importlib.util.spec_from_file_location(
    "ab", os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "actions-ab", "harness", "tally.py"))
ab = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ab)
ab.ARMS = ["toolbox", "gitlab"]

if __name__ == "__main__":
    sys.argv = sys.argv[:2]
    ab.main()
