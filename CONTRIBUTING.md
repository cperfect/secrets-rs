# Contributing — Rust
> This guide applies equally to AI agents and humans, unless otherwise stated.

> The key words "MUST", "SHOULD", "MAY", etc. are used as defined in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

## Code Style

1. **MUST follow the [Rust Style Guide](https://doc.rust-lang.org/style-guide/index.html)** — `rustfmt` handles most of this automatically.
2. **SHOULD follow the [Rust API Design Guidelines](https://rust-lang.github.io/api-guidelines/)** for internal libraries and APIs.

Run `cargo fmt --all` before committing. CI will reject unformatted code.

## Comments

Doc-comments (`///`) **MUST** be provided for all public functions, types, and modules.

Additional inline comments **SHOULD** be added for clarity when needed — e.g. complex algorithms, non-obvious choices, or use of a library where constraints are not self-evident. Such comments should explain *why*, not just *what*.

## Developing

A dev container configuration is provided for VS Code and compatible editors — this is the recommended way to get a consistent environment. The container pins the following versions:

| Component | Version | Defined in |
|-----------|---------|------------|
| Rust | 1.95.0 | Cargo.toml |

```bash
# Build the workspace
cargo build

# Run the full test suite
cargo test

# Check formatting (must produce no output)
cargo fmt --all -- --check

# Run static analysis (must produce no warnings)
cargo clippy --all-targets -- -D warnings
```

### Pre-commit Checklist

Run these before every commit and fix any issues found:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Rust Development Guidelines

> The key words "MUST", "MUST NOT", "REQUIRED", "SHALL", "SHALL NOT", "SHOULD", "SHOULD NOT", "RECOMMENDED", "MAY", and "OPTIONAL" in this document are to be interpreted as described in [RFC 2119](https://www.ietf.org/rfc/rfc2119.txt).

1. **The project MUST follow the Rust Style Guide**: https://doc.rust-lang.org/style-guide/index.html (`rustfmt` should take care of most of this).
1. **Errors SHOULD follow the NRC Error Design Guidelines**: https://nrc.github.io/error-docs/error-design/index.html
2. **Internal API/Libraries SHOULD follow the Rust Lang API Design Guidelines**: https://rust-lang.github.io/api-guidelines/


### Perform Local Checks before committing
(unless no rust code was changed)
- `cargo fmt --check`
- `cargo clippy`
- `cargo test`

and fix any issues found

#### Other Guidelines
1. **Code SHOULD be safe & secure** Code should implement secure code practices, e.g. [OWASP](https://owasp.org/www-project-secure-coding-practices-quick-reference-guide/stable-en/02-checklist/05-checklist)
1. **Systems SHOULD be deterministic and synchronous as far as possible** Non-determistic and asynchronous behavious should be isolated and clearly named and documented
1. **Interfaces MUST be developed Schema First** This includes both functions and models.
1. **Code MUST include standard doc-comments for public/exported elements** based on the landguage used.
1. **Additional inline comments SHOULD be added for clarity** e.g. for complex code, use of non-standard or inconsistent, patterns, or the intent is not clear from the name, or the code relates to a library or external service where constraints and usage are not obvious. 
1. **Commits SHOULD conform to the Conventional Commits standard** see https://www.conventionalcommits.org/en/v1.0.0/#summary
1. **The Git Workflow WILL BE Trunk Based**
1. **Shell scripting SHOULD be minimised** — prefer CLI tooling, Makefiles, or application code over shell scripts. Shell scripts are hard to test, port, and maintain.
1. **No error should just be ignored** - either it is worth noticing or it isn't an error
### Testing
1. **Tests MUST be automated**
1. **Tests SHOULD be reliable** flakey tests are broken tests. Either fix what is being tested of fix the tests.
1. **Overall Test Coverage SHOULD aim towards the Test Diamond** As opposed to the Test Pyramid. That is Integration Teating should have the most coverage (ideally tending to 100%), and UI and unit testing are less important.


