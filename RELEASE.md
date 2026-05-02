# Release process

Releases are created manually and published to crates.io automatically via GitHub Actions.

## Overview

Two workflows handle the pipeline:

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `01-Validate.yml` | Push / PR | Runs fmt, clippy, and tests |
| `02-CodeQL.yml` | Push / PR / weekly | Runs CodeQL security analysis |
| `03-Release.yml` | Manual | Creates a GitHub Release and publishes both crates to crates.io |

`03-Release.yml` runs in the `release` environment, which is restricted to the `main` branch.

## Before releasing

1. Ensure all changes are merged to `main` and `01-Validate.yml` and `02-CodeQL.yml` are green.
2. Update `version` in both `Cargo.toml` (root) and `macros/Cargo.toml` to the new version.
3. Add a `## [x.y.z] — YYYY-MM-DD` section to `CHANGELOG.md`.
4. Commit and push those changes to `main`.

## Creating a release

1. Go to **Actions → Release** (`03-Release.yml`) in the GitHub UI.
2. Click **Run workflow** (branch: `main`) then **Run workflow**.

The workflow will:

1. Read the version from `Cargo.toml`.
2. Create a GitHub Release tagged `v<version>` with auto-generated release notes.
3. Publish `secrets-rs-macros` to crates.io.
4. Wait 20 seconds for the registry to index it.
5. Publish `secrets-rs` to crates.io.

## Credentials

`CARGO_REGISTRY_TOKEN` must be set as a secret on the `release` environment in GitHub (**Settings → Environments → release**). The token is a crates.io API token scoped to publish both crates.
