# Security Policy

## Supported versions

| Version | Supported |
|---------|-----------|
| 0.1.x   | ✓         |

## Reporting a vulnerability

**Please do not open a public GitHub issue for security vulnerabilities.**

Send a private report to **cyruskakkar@gmail.com** with the subject line
`[crement] Security vulnerability`.  Include:

- A description of the vulnerability and its potential impact.
- Steps to reproduce or a minimal proof-of-concept.
- The version(s) of `crement` affected.
- Any mitigations or workarounds you are aware of.

You will receive an acknowledgement within **72 hours** and a resolution
timeline within **7 days**.

## Scope

`crement` is a procedural macro crate.  Its attack surface is limited to
**compile-time** code generation: malicious input can, at worst, produce a
compile error or unexpected generated code, but cannot execute arbitrary code
at runtime in a user's binary without the user's explicit consent (i.e. by
running `cargo build`).

Vulnerabilities in the compile-time expansion — such as macro hygiene bypass,
unexpected token injection, or generation of memory-unsafe code — are
considered in scope and will be treated seriously.
