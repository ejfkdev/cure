# Release checklist

`cure` ships six raw binaries from a tag (`vX.Y.Z`) and publishes 15 crates
to crates.io in dependency order.

1. **Bump the version everywhere**: `scripts/bump-version.sh X.Y.Z`
   (workspace package version, every `cure-*` dependency requirement, and
   `Cargo.lock`). Never publish a version that already exists on crates.io
   with different code.
2. **Update `CHANGELOG.md`**: move the current `[Unreleased]` entries under
   `## [X.Y.Z] - YYYY-MM-DD` and refresh the compare links.
3. **Verify locally**:
   - `cargo test --workspace --release`
   - `cargo publish --dry-run -p cure-engine` and `-p cure-tree` (the two
     crates with no `cure-*` dependency; later crates can only be dry-run
     after their upstream version is on crates.io)
   - Optional full corpora: the Test262 six-layer examples and the npm
     corpus checks documented in README.
4. **Commit** the version bump and docs.
5. **Tag and push**:
   ```bash
   git tag -a vX.Y.Z -F - <<'EOF'
   cure X.Y.Z

   <release highlights and verification numbers>
   EOF
   git push origin main
   git push origin vX.Y.Z
   ```
   The tag subject and body become the GitHub Release description. The
   release workflow builds all six targets, stages raw
   `cure-vX.Y.Z-<target>` assets, and runs `scripts/publish.sh` for
   crates.io (15 crates, idempotent on re-runs).
6. **Preconditions**: repository secret `CARGO_REGISTRY_TOKEN` must be valid
   (crates.io token with `publish-new` + `publish-update`); CI must be green
   on the tagged commit.
7. **Post-release**:
   - `cargo install cure-cli` and `cargo binstall cure-cli` resolve.
   - `cure version` on each release asset reports `X.Y.Z`.
   - GitHub assets are raw binaries named with the full target triple
     (`cure-vX.Y.Z-x86_64-unknown-linux-gnu`, etc.).
