# Afterburn

One-shot agent for cloud-like platforms. Retrieves instance metadata (attributes, SSH keys, hostname, network config) from provider-specific endpoints and applies it to the local system. Used on Fedora CoreOS and similar immutable Linux distributions.

## Tech Stack

- **Language**: Rust (edition 2021, MSRV 1.84.1)
- **CLI**: clap v4 (derive)
- **HTTP**: reqwest (blocking)
- **Serialization**: serde, serde_json, serde_yaml, serde-xml-rs
- **Logging**: slog (structured logging)
- **Error Handling**: anyhow
- **Testing**: Built-in #[test] + mockito for HTTP mocking
- **Build**: Cargo + GNU Make
- **Packaging**: RPM (Fedora)

## Architecture

```
src/
  main.rs              # Entry point
  cli/                 # CLI argument parsing (clap derive)
  metadata.rs          # Top-level provider dispatch
  network.rs           # Network interface/config types
  providers/           # Cloud provider implementations (one subdir each)
    mod.rs             # MetadataProvider trait definition
    aws/ gcp/ microsoft/ openstack/ ...  # 22 providers
  retry/               # HTTP retry/client logic
  util/                # Utilities (cmdline, DHCP, etc.)
dracut/30afterburn/    # Dracut initramfs module + systemd units
systemd/               # Systemd service unit templates
tests/
  fixtures/            # Test fixture data (JSON/YAML metadata)
  tmt/                 # TMT integration test plans
docs/                  # Jekyll documentation (GitHub Pages)
```

Each provider implements the `MetadataProvider` trait (`src/providers/mod.rs`) with default no-op methods for: `attributes()`, `hostname()`, `ssh_keys()`, `networks()`, `boot_checkin()`, etc.

## Build Commands

- `make` - Build systemd units + `cargo build --release`
- `make test` - `cargo test --all-targets --release`
- `make fmt` - `cargo fmt -- --check -l`
- `make lint` - `cargo clippy --all-targets -- -D warnings`
- `make validate-fixtures` - Validate test fixtures against upstream schemas
- `make install` - Install binary, systemd units, and dracut modules

## Code Style

- Default `rustfmt` formatting (no custom config)
- `clippy` with `-D warnings` (all warnings are errors)
- Apache 2.0 license headers at top of source files
- `slog` for logging (not `log` or `tracing`)
- `anyhow` with `.context()` for error handling
- Module organization: `mod.rs` pattern for provider directories

## Testing

- **Unit tests**: Inline `#[cfg(test)] mod tests` blocks within source files
- **Mock tests**: Separate `mock_tests.rs` per provider using `mockito` for HTTP mocking
- **Fixture tests**: `tests.rs` files loading data from `tests/fixtures/` (kubevirt, proxmoxve)
- **Fixture validation**: `python3 tests/fixtures/validate.py` checks against upstream schemas
- Run `cargo test --all-targets` before submitting changes

## Commit Conventions

**Format**: `<subsystem>: <What changed>`

Examples:
- `kubevirt: Support static gateway and DNS with DHCP`
- `microsoft/azure: Fix SharedConfig parsing of XML attributes`
- `docs/release-notes: update for release 5.10.0`
- `cargo: Afterburn release 5.10.0`

**Style**: Subject max 70 chars, body wrapped at 80 chars. Imperative tense preferred. Lowercase subsystem, capitalized description.

## Important Rules

- **Do not edit CI workflows** (`.github/workflows/*.yml`, `.copr/Makefile`) -- synced from `coreos/repo-templates`
- **Every PR must update `docs/release-notes.md`** (enforced by CI; use `skip-notes` label to override)
- **DCO required**: Contributors agree to the Developer Certificate of Origin
- **Fork-and-PR workflow** from topic branches based on `main`
- Release commits: `cargo: Afterburn release {{version}}` (signed)

## Resources

- [Supported platforms](docs/platforms.md)
- [Contributing guide](docs/contributing.md)
- [Development docs](docs/development.md)
- [Attributes reference](docs/usage/attributes.md)
