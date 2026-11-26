"""YAML 1.2 core-schema loader for the reference tools.

PyYAML implements YAML 1.1, where a plain timestamp (`2026-09-02`) is a
datetime. Seikika parses manifests with serde_norway and keeps that scalar a
string, and yamark (D95) emits YAML 1.2, where it is a string. Dropping the
timestamp resolver makes these tools agree with both.
"""

import yaml

_TIMESTAMP = "tag:yaml.org,2002:timestamp"


def loader():
    class Loader(yaml.SafeLoader):
        pass

    Loader.yaml_implicit_resolvers = {
        key: [(tag, regexp) for tag, regexp in resolvers if tag != _TIMESTAMP]
        for key, resolvers in yaml.SafeLoader.yaml_implicit_resolvers.items()
    }
    return Loader


def load(path):
    with open(path) as handle:
        return yaml.load(handle, loader())
