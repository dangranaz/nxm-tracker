# Security Policy

## Supported Versions

The following versions receive security updates:

| Version | Supported |
|---------|-----------|
| 0.7.2+  | ✅ Yes    |
| < 0.7.2 | ❌ No     |

## Reporting a Vulnerability

If you discover a security vulnerability in `nxm-tracker`, please report it before making it public.

### How to report

Open a private issue on GitHub or send an email to the maintainer with the following details:

- Description of the vulnerability
- Steps to reproduce (input, MCP tool arguments, configuration)
- Affected crate version (`Cargo.toml` `version`)
- Potential impact (e.g. code execution, data exposure, denial of service)
- Suggested fix or workaround, if any

### What not to do

- Do not open public issues for security vulnerabilities before receiving a response from the maintainer.
- Do not disclose the vulnerability on social media, forums, or other public channels before a patch is released.

## Disclosure Policy

| Phase | Expected timeframe |
|-------|-------------------|
| Confirm receipt of report | within 7 days |
| Evaluation and classification | within 14 days |
| Patch or workaround release | within 30 days (or otherwise agreed with the reporter) |
| Advisory publication | after patch release, with credit to the reporter if desired |

## Security Best Practices Adopted

This section summarizes the security choices already present in the crate, consistent with the rules in `.pi/rules/security.md`.

### Input validation

All MCP tool arguments are validated before use:

- `id`, `session_id` must be valid integers (`i64`)
- `name` must not be empty
- `status` must be one of `open`, `paused`, `closed` (via `SessionStatus::try_from`)
- `design_phase_complete` must be a boolean

### Secret handling

- No secrets (passwords, tokens, keys) are stored in source code or in the database.
- Configuration is read from `~/.nxmconfig` (user-owned file).
- Logs are written in JSON to `~/.nxmconfig/logs/nxm-tracker.log` and stderr; they do not include sensitive data by default.

### Dependency auditing

The project runs as part of `just check`:

- `cargo audit` — verifies known vulnerabilities in dependencies
- `cargo deny check` — verifies licenses and dependency policies

### Unsafe code

The crate does not use `unsafe` blocks. All operations are in safe Rust.

### Data protection

- The database is a local SQLite file (`~/.nxm/shared-db`), not directly exposed on the network.
- The default MCP transport is stdio (JSON-RPC over standard input/output).
- If used in HTTP/HTTPS mode, transport is handled by `nxm-mcp-server` with its own transport guarantees.

## MCP Tools Security

The eight MCP tools (`create_session`, `get_session`, `list_sessions`, `update_session`, `close_session`, `set_design_phase_complete`, `create_feature_session`, `list_feature_sessions`) always return an envelope with `isError` on failure, never a JSON-RPC protocol error. Errors contain descriptive messages but do not leak internal stack traces or filesystem details.

## References

- Project security rules: `.pi/rules/security.md` (source of truth: `~/AiProjects/nexum/docs/configRust/rules/security.md`)
- License: PolyForm Noncommercial 1.0.0 (`LICENSE.md`)
