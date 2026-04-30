# Release process

Releases are created manually and published to crates.io automatically via GitHub Actions.

## Overview

Three workflows handle the pipeline:

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `01-Validate.yml` | Push / PR | Runs fmt, clippy, and tests |
| `02-Release.yml` | Manual | Creates a GitHub Release from `main` |
| `03-Publish.yml` | GitHub Release published | Publishes both crates to crates.io |

Both `02-Release.yml` and `03-Publish.yml` run in the `release` environment, which is restricted to the `main` branch.

## Before releasing

1. Ensure all changes are merged to `main` and `01-Validate.yml` is green.
2. Update `version` in both `Cargo.toml` (root) and `macros/Cargo.toml` to the new version.
3. Add a `## [x.y.z] — YYYY-MM-DD` section to `CHANGELOG.md`.
4. Commit and push those changes to `main`.

## Creating a release

1. Go to **Actions → Release** in the GitHub UI.
2. Click **Run workflow** (branch: `main`) then **Run workflow**.

The workflow reads the version directly from `Cargo.toml`, creates a GitHub Release tagged `v<version>` with auto-generated release notes, and immediately triggers `03-Publish.yml`.

## What happens automatically

`03-Publish.yml` will:

1. Verify the release tag matches the `Cargo.toml` version.
2. Publish `secrets-rs-macros` to crates.io.
3. Wait 20 seconds for the registry to index it.
4. Publish `secrets-rs` to crates.io.

## Credentials

`CARGO_REGISTRY_TOKEN` must be set as a secret on the `release` environment in GitHub (**Settings → Environments → release**). The token is a crates.io API token scoped to publish both crates.
