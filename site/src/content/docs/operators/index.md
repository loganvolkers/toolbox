---
title: Build a portal
description: Configure standards in Git, check changes with fixtures, and serve a catalog.
sidebar:
  order: 1
---

A portal gives developers and agents a place to find software, check ownership, and read standards results. As an operator, you own its configuration, data sources, and deployment.

## Choose where to start

| Goal | Guide |
|---|---|
| Score one checkout and catch regressions in CI | [Gate pull requests](/operators/ci/) |
| Learn how a running catalog works | [Explore a portal](/start/portal/) |
| Define catalog data and rules | [Portal manifest reference](/operators/manifest/) |
| Review a configuration change | [Change a portal](/operators/portal/) |
| Run a shared service | [Deploy and operate a portal](/operators/deploy/) |
| Configure access for people and integrations | [Authentication and authorization](/operators/access/) |
| Define a self-service action | [Define a self-service action](/operators/actions/) |
| Offer actions from several repositories | [Federate repository actions](/operators/federate-actions/) |
| Publish a snapshot with no portal server | [Generate and host a static report](/operators/publish/) |
| Track a rollout or update a standard | [Track progress and change a standard](/operators/standards/) |
| Collect a human or agent judgment | [Assess a catalog entity](/operators/assessments/) |

## Keep configuration and tests together

Put `seikika.yaml` in Git. This **manifest** declares which connectors provide data, how to map their records to catalog entities, and which rules to score. Include packages to reuse standards. Keep representative fixtures and tests with the manifest so reviewers can check what a change does before deploying it.

Run `seikika validate` to catch configuration errors, `seikika test` to check fixture expectations, and `seikika plan --local --base` to see how two revisions score the same inputs. Then verify the deployed portal using its readiness endpoint and a sample read. The [release and upgrade guide](/start/versions/) walks through these checks.

For credentials, use [connector secrets](/operators/secrets/). For a custom data source, see [Write a connector](/authors/connectors/).
