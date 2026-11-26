# Seikika

Seikika is an open-source framework for building an internal developer portal in code.
Use it to check standards in one repository or to run a shared catalog with service
ownership, scorecards, and self-service actions for developers and agents.

## Get started

| If you want to | Start here |
| --- | --- |
| Check one repository | [Follow the repository quickstart](https://seikika.com/start/quickstart/) to record a baseline and catch a broken link. A baseline saves the results you compare later. |
| Run a catalog locally | [Explore a portal](https://seikika.com/start/portal/) to look up a service, see who owns it, and inspect its score. |
| Bring GitHub or other data into a portal | [Browse the connector directory](https://seikika.com/connectors/) for setup guides. |
| Find the right CLI and connector | [Install Seikika](https://seikika.com/start/install/) for release and source build options. |

See [versions and upgrades](https://seikika.com/start/versions/) for compatibility
and the [releases](https://github.com/seikika/seikika/releases) for available binaries.

## Use or extend Seikika

- **Portal operators** can [enforce standards in CI](https://seikika.com/operators/ci/)
  or [deploy a shared portal](https://seikika.com/operators/deploy/).
- **Package authors** can [share standards and write connectors](https://seikika.com/authors/) or
  browse the [first-party standards packages](packages/README.md).
- **Developers and agents** can [read the catalog and request actions](https://seikika.com/users/).
- **Maintainers** can [build and check a source checkout](https://seikika.com/start/run/).
  See the [contribution workflow](AGENTS.md), [workspace crates](crates/README.md),
  and [design and research index](docs/README.md).

## Project status

Seikika v1 is in development. Repository checks, catalog reads, and federated
self-service actions are implemented. Durable action runs and action pages in the
portal UI are planned. See the [implementation contract](docs/v1-spec.md) for details.

Seikika [scores this repository](https://seikika.com/report/) using its own standards.
The [developer experience benchmarks](portal/benchmarks/) record how agents fare
on open-source tasks with Seikika and other tools.

## Help and license

Start with [troubleshooting](https://seikika.com/start/troubleshooting/) or
[open an issue](https://github.com/seikika/seikika/issues) with your version and
reproduction steps. Seikika is available under the [MIT license](LICENSE-MIT).
