# Security policy

Mulu opens files that may come from anywhere, so we treat security problems as urgent.

## Reporting a problem

Please **do not open a public issue**. Report it privately through GitHub:

1. Go to the **Security** tab of this repository.
2. Click **Report a vulnerability**.
3. Describe the problem, the affected version and, if possible, a file or steps that reproduce it.

We will acknowledge the report within 7 days and keep you informed until it is resolved. With your permission, we will credit you in the release notes.

## Supported versions

No version has been released yet. Once releases begin, security fixes will go into the latest version.

## What is in scope

- Files that crash Mulu, hang it, use excessive memory, or escape the isolated reading process.
- Any way to make Mulu run code embedded in a document.
- Any way to make Mulu change, move or delete a user's file without confirmation.
- Any network connection Mulu makes without the user's consent.
- Problems in the build and release pipeline, including installers that do not match the public source code.

## How Mulu reduces risk

- Files are parsed in a separate process with time and memory limits.
- Rust, a memory-safe language, is used wherever possible.
- Dependencies are few, pinned, and monitored for known vulnerabilities.
- Installers are built only by GitHub Actions from this public repository, with a build provenance attestation and published checksums.
- Automated workflows run with minimal permissions, and third-party actions are pinned to an exact commit.
- An automated test fails if Mulu attempts any network connection.
- Mulu never runs macros, scripts or other code embedded in documents.
- Mulu never asks for administrator privileges.

See decision D12 in [DECISIONS.md](DECISIONS.md) for the full list.
