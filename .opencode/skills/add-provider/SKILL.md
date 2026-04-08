---
name: add-provider
description: Scaffold a new cloud/platform provider for Afterburn with all required files, wiring, docs, and tests
---

# Add Provider

Scaffold a new cloud/platform provider for the Afterburn metadata agent, including the Rust implementation, mock tests, documentation, and systemd/dracut service entries.

## What it does

1. Gathers provider details from the user (name, metadata source, features, attributes)
2. Creates the provider Rust module (`src/providers/{name}/mod.rs`)
3. Creates mock tests (`src/providers/{name}/mock_tests.rs`)
4. Wires the provider into the module tree and dispatch table
5. Updates all documentation files (platforms, attributes, release notes)
6. Updates systemd/dracut service files based on supported features
7. Verifies the build compiles, tests pass, and lints are clean

## Prerequisites

- Rust toolchain installed (`cargo`, `rustfmt`, `clippy`)
- The repository builds successfully before starting (`cargo build --all-targets`)

## Usage

```bash
/add-provider
/add-provider --name linode
/add-provider --name cherry --type imds
```

## Workflow

### Step 1: Gather Provider Information

Ask the user for the following information. If any arguments were provided via the command, use those instead of asking.

Use the question tool to ask:

**Question 1: Provider name**
- What is the platform ID for this provider? (lowercase, no spaces, e.g., `linode`, `cherry`, `vultr`)
- This will be used as the directory name under `src/providers/` and the match arm in `fetch_metadata()`

**Question 2: Provider struct name**
- What should the Rust struct be named? (e.g., `LinodeProvider`, `CherryProvider`)
- Default: derive from provider name by capitalizing and appending `Provider`

**Question 3: Metadata source type**
- Is this an IMDS (HTTP metadata service) or config drive (mounted filesystem) provider?
- Options: `imds`, `configdrive`

**Question 4 (if IMDS): Base URL**
- What is the metadata service base URL?
- Common pattern: `http://169.254.169.254/metadata/v1`

**Question 5 (if IMDS): Authentication**
- What authentication method does the metadata service use?
- Options: `none`, `token-header` (static header like OracleCloud's `Bearer Oracle`), `imdsv2` (token exchange like AWS)

**Question 6: Supported features**
- Which features does this provider support? (multi-select)
- Options: `attributes` (always), `ssh-keys`, `hostname`, `boot-checkin`, `network-config`

**Question 7: Attributes**
- What attributes does this provider expose?
- For each attribute, ask: attribute name (e.g., `HOSTNAME`, `INSTANCE_ID`, `REGION`) and the metadata endpoint/path to fetch it from
- Attribute names will be prefixed with `AFTERBURN_{UPPER_PROVIDER_NAME}_`

### Step 2: Validate Inputs

Before creating any files, verify:

1. Read `src/providers/mod.rs` and confirm no `pub mod {name};` exists
2. Read `src/metadata.rs` and confirm no `"{name}"` match arm exists in `fetch_metadata()`
3. Confirm no directory exists at `src/providers/{name}/`

If any conflict is found, report the conflict and stop.

### Step 3: Create Provider Module

Create `src/providers/{name}/mod.rs`.

**For IMDS providers**, use the UpCloud provider (`src/providers/upcloud/mod.rs`) as the primary reference. Key patterns:

```rust
// Structure:
// 1. Apache 2.0 license header
// 2. Module doc comment with platform ID and metadata docs URL
// 3. Imports: anyhow::Result, openssh_keys::PublicKey, slog_scope::error, std::collections::HashMap
// 4. Import crate::providers::MetadataProvider and crate::retry
// 5. #[cfg(test)] mod mock_tests;
// 6. Provider struct with retry::Client field
// 7. impl block with try_new(), endpoint_for(), and helper methods
// 8. impl MetadataProvider with attributes(), hostname(), ssh_keys()
```

Key rules:
- The `endpoint_for()` helper constructs the full URL from base URL + endpoint name
- Use `retry::Raw` for plain text endpoints, `retry::Json` for JSON endpoints
- For providers with auth headers, use `.header()` on the request builder
- SSH keys: iterate lines, parse each with `PublicKey::parse()`, log errors with `slog_scope::error!`
- Hostname: return `Ok(None)` if empty or missing, `Ok(Some(hostname))` otherwise
- Attributes: use `HashMap::with_capacity(N)` where N is the number of attributes

**For config drive providers**, use the ProxmoxVE provider (`src/providers/proxmoxve/`) as reference. These are more complex and variable -- adapt based on the specific config drive format.

### Step 4: Create Mock Tests

Create `src/providers/{name}/mock_tests.rs`.

Use the UpCloud mock tests (`src/providers/upcloud/mock_tests.rs`) as the primary reference. Every provider should have at minimum:

1. **`test_hostname()`** (if hostname supported):
   - Test successful fetch (200 with body)
   - Test 404 returns `None`
   - Test empty body returns `None`
   - Test 503 returns error
   - Test connection error (server reset) returns error

2. **`test_pubkeys()`** (if SSH keys supported):
   - Test with 2 SSH keys
   - Verify key count and comment fields
   - Test connection error returns error

3. **`test_attributes()`**:
   - Mock all attribute endpoints with test values
   - Verify all attributes are returned with correct keys
   - Test connection error returns error

Use `mockito::Server` for all HTTP mocking. Create the provider with `try_new().unwrap()` and override the client:
```rust
provider.client = provider.client.max_retries(0).mock_base_url(server.url());
```

### Step 5: Wire into Module Tree

Edit `src/providers/mod.rs`:
- Add `pub mod {name};` in **alphabetical order** among the existing module declarations

### Step 6: Register in Metadata Dispatch

Edit `src/metadata.rs`:
- Add `use crate::providers::{name}::{ProviderStruct};` in the import block (alphabetical order)
- Add match arm `"{platform_id}" => box_result!({ProviderStruct}::try_new()?),` in `fetch_metadata()` (alphabetical order)

### Step 7: Update Documentation

#### 7a. `docs/platforms.md`
Add entry in **alphabetical order** among the platform list:
```markdown
* {platform_id}
  - Attributes
  - Hostname        (if supported)
  - SSH Keys        (if supported)
  - Boot check-in   (if supported)
  - Network configuration (if supported)
```

#### 7b. `docs/usage/attributes.md`
Add entry in **alphabetical order** with all attributes:
```markdown
* {platform_id}
  - AFTERBURN_{UPPER_NAME}_ATTRIBUTE_1
  - AFTERBURN_{UPPER_NAME}_ATTRIBUTE_2
  ...
```

#### 7c. `docs/release-notes.md`
Add under "Major changes:":
```markdown
- Add support for {Provider Display Name}
```

### Step 8: Update Service Files

Based on supported features, add `ConditionKernelCommandLine=|ignition.platform.id={platform_id}` to the appropriate service files. Insert in **alphabetical order** among existing entries.

#### 8a. SSH Keys (if supported)
Edit `systemd/afterburn-sshkeys@.service.in`:
```ini
ConditionKernelCommandLine=|ignition.platform.id={platform_id}
```

#### 8b. Hostname (if supported)
Edit `dracut/30afterburn/afterburn-hostname.service`:
```ini
ConditionKernelCommandLine=|ignition.platform.id={platform_id}
```

#### 8c. Boot Check-in (if supported)
Edit `systemd/afterburn-checkin.service`:
```ini
ConditionKernelCommandLine=|ignition.platform.id={platform_id}
```

### Step 9: Verify

Run the following commands sequentially and fix any issues:

```bash
cargo fmt
cargo build --all-targets
cargo test --all-targets
cargo clippy --all-targets -- -D warnings
```

If any command fails:
1. Read the error output carefully
2. Fix the issue in the relevant file
3. Re-run from the failing command

### Step 10: Summary

After all steps complete, present a summary:

```
Provider scaffold complete!

Files created:
  - src/providers/{name}/mod.rs
  - src/providers/{name}/mock_tests.rs

Files modified:
  - src/providers/mod.rs
  - src/metadata.rs
  - docs/platforms.md
  - docs/usage/attributes.md
  - docs/release-notes.md
  - systemd/afterburn-sshkeys@.service.in (if SSH keys)
  - dracut/30afterburn/afterburn-hostname.service (if hostname)
  - systemd/afterburn-checkin.service (if boot check-in)

Verification:
  - cargo build: PASS
  - cargo test: PASS
  - cargo clippy: PASS

Next steps:
  1. Fill in the actual metadata endpoint logic in mod.rs
  2. Update mock tests with realistic test data
  3. Run `cargo test` to verify
  4. Submit a PR with commit message: "{name}: implement {name} provider"
```

## Checklist Coverage

This skill automates the following from the contributor workflow:

- [x] Create provider module with MetadataProvider trait implementation
- [x] Create mock tests for hostname, SSH keys, and attributes
- [x] Register module in `src/providers/mod.rs`
- [x] Register provider in `src/metadata.rs` dispatch table
- [x] Update `docs/platforms.md` with supported features
- [x] Update `docs/usage/attributes.md` with attribute list
- [x] Update `docs/release-notes.md` with release note
- [x] Update `systemd/afterburn-sshkeys@.service.in` (if SSH keys supported)
- [x] Update `dracut/30afterburn/afterburn-hostname.service` (if hostname supported)
- [x] Update `systemd/afterburn-checkin.service` (if boot check-in supported)
- [x] Run `cargo build`, `cargo test`, `cargo clippy`

## What's NOT covered

- Implementing the actual metadata fetching logic (provider-specific API calls, JSON parsing, etc.)
- Creating test fixtures for config drive providers
- Testing against a real cloud environment
- The PR review and merge process
- Release notes for subsequent releases

## Example Output

When you run `/add-provider`, the interaction looks like:

```
> /add-provider

What is the platform ID? > mycloud
What should the struct be named? > MyCloudProvider
IMDS or config drive? > imds
Base URL? > http://169.254.169.254/metadata/v1
Authentication? > none
Features? > attributes, ssh-keys, hostname
Attributes? > HOSTNAME (hostname), INSTANCE_ID (instance_id), REGION (region)

Creating provider scaffold...

Created: src/providers/mycloud/mod.rs (124 lines)
Created: src/providers/mycloud/mock_tests.rs (106 lines)
Modified: src/providers/mod.rs (+1 line)
Modified: src/metadata.rs (+2 lines)
Modified: docs/platforms.md (+4 lines)
Modified: docs/usage/attributes.md (+4 lines)
Modified: docs/release-notes.md (+2 lines)
Modified: systemd/afterburn-sshkeys@.service.in (+1 line)
Modified: dracut/30afterburn/afterburn-hostname.service (+1 line)

Running verification...
  cargo fmt: PASS
  cargo build: PASS
  cargo test: PASS
  cargo clippy: PASS

Provider scaffold complete!
```

## Provider Architecture Variants

There are two main architectural patterns. Read the referenced providers to match the right pattern:

### Pattern A: IMDS (Instance Metadata Service)
Used by: upcloud, oraclecloud, akamai, aws, gcp, digitalocean, exoscale, aliyun, hetzner, scaleway, vultr, packet
- Fetches metadata via HTTP from a well-known IP (usually `169.254.169.254`)
- May require token-based authentication (IMDSv2 style)
- Uses `retry::Client` for HTTP requests
- Provider struct holds the client (or pre-fetched data)

### Pattern B: Config Drive
Used by: proxmoxve, powervs, ibmcloud-classic, cloudstack-configdrive
- Reads metadata from a mounted filesystem (ISO/config drive)
- Parses JSON or YAML files from the mount point
- May include network configuration
- Often includes test fixtures in `tests/fixtures/{name}/`

## Reference Examples

### UpCloud (commit `c8cc721`) -- simplest IMDS pattern
- 9 files, +245 lines
- Plain text endpoints, no auth, `retry::Client` with `return_on_404(true)`
- Fetches individual endpoints per attribute (`/metadata/v1/{attr}`)

### OracleCloud (commit `d4f8031`) -- IMDS with JSON + auth
- 8 files, +332 lines
- Single JSON endpoint with `Authorization: Bearer Oracle` header
- Serde deserialization with `#[serde(rename_all = "camelCase")]`
- `try_new_with_client()` pattern for test injection

### Akamai (commit `bacbf84`) -- IMDS with token auth, many attributes
- 8 files, +411 lines
- Token-based authentication, 13 attributes
- No hostname service entry

### ProxmoxVE (commit `a92c78d`) -- config drive with network config
- 27 files, +707 lines
- Reads from mounted filesystem, YAML parsing
- Includes test fixtures in `tests/fixtures/proxmoxve/`

## References

- Provider trait: `src/providers/mod.rs:190` (`MetadataProvider` trait)
- Dispatch table: `src/metadata.rs:54` (`fetch_metadata()` function)
- Contributing guide: `docs/contributing.md`
