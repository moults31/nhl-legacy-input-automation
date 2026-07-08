# Security Policy

## Supported Versions

Only the latest commit on `main` is supported. This project does not have formal releases.

## Reporting a Vulnerability

Email a summary to the repository maintainer. Do not open a public issue.

## Scope

This project is a virtual Xbox controller input automation tool for NHL Legacy Edition. Security concerns fall into two categories:

1. **Malicious scripts**: The tool executes user-provided [Rhai](https://rhai.rs) scripts with access to the virtual controller. Rhai is sandboxed by default. If you find a way to escape the sandbox, please report it.

2. **Dependencies**: We run `cargo audit` and `cargo deny` in CI on every push to catch known vulnerabilities in dependencies.
