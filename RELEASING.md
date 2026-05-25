# Release checklist

Follow these steps when cutting a new release of `crement`.

## Before the release

1. **Update `CHANGELOG.md`**
   - Move items from `[Unreleased]` into a new dated section
     `## [X.Y.Z] - YYYY-MM-DD`.
   - Add the diff link at the bottom.

2. **Bump the version in `Cargo.toml`**
   ```
   version = "X.Y.Z"
   ```

3. **Run the full test suite locally**
   ```bash
   cargo test
   cargo clippy -- -D warnings
   cargo fmt --check
   cargo doc --no-deps
   ```

4. **Update trybuild stderr snapshots if needed**
   ```bash
   TRYBUILD=overwrite cargo test --test 06_ui
   ```

5. **Dry-run publish**
   ```bash
   cargo publish --dry-run
   ```

6. **Commit**
   ```bash
   git add Cargo.toml CHANGELOG.md tests/ui/
   git commit -m "chore: release v X.Y.Z"
   ```

## Publishing

7. **Tag the commit** (this triggers the publish workflow automatically)
   ```bash
   git tag vX.Y.Z
   git push origin main --tags
   ```

8. **Verify the GitHub Actions run**
   - The `publish.yml` workflow will:
     1. Run `cargo test`.
     2. Check the tag matches `Cargo.toml`.
     3. Run `cargo publish` with the `CARGO_REGISTRY_TOKEN` secret.
     4. Create a GitHub release with the changelog excerpt.

9. **Confirm on crates.io** that the new version appears at
   `https://crates.io/crates/crement`.

## Setting up the crates.io token (first time)

1. Log in at [crates.io](https://crates.io) and create a token under
   *Account Settings → API Tokens*.  Scope: `publish-new` and
   `publish-update`.
2. Add it as a repository secret named `CARGO_REGISTRY_TOKEN` in
   *GitHub → Settings → Secrets and variables → Actions*.

## Semver policy

- **Patch** (0.0.X): bug fixes, documentation, no API changes.
- **Minor** (0.X.0): new macros or features, MSRV bumps.
- **Major** (X.0.0): breaking changes to existing macro behaviour.
