# Publishing a release

1. Update the changelog. If there is an "Unreleased" section, put the release version and date there.
2. Update the version in `yerpc/Cargo.toml`, `yerpc-derive/Cargo.toml` and `typescript/package.json`.
3. Make a commit titled "Prepare for XXX release" with a changelog and version update on the `main` branch.
4. Push the commit.
5. Create annotated tag prefixed with `v`, e.g. `v0.6.3`.
6. Push the tag.
7. Run `scripts/publish.sh` for a dry run.
8. Run `scripts/publish.sh publish`.
