# Dependency and asset record

The workspace pins Rust 1.95.0 in `rust-toolchain.toml`, pins Bevy 0.19.1 in
`crates/app/Cargo.toml`, and retains exact resolved crate versions and checksums
in `Cargo.lock`. The only external direct application dependency is
[Bevy 0.19.1](https://crates.io/crates/bevy/0.19.1), from crates.io, licensed
MIT OR Apache-2.0. The core uses Serde for project serialization and Schemars
for schema generation; Proptest and jsonschema are test-only dependencies.
The app also uses Proptest as a test-only dependency for sprite property tests. The
tools crate uses Schemars, serde_json, and jsonschema for schema generation and
validation.

[The dependency inventory](dependency-licenses.csv) lists the origin and
declared license expression for all 576 registry packages in Cargo's locked
cross-platform resolution, including packages for targets not used by T01/T02.
It was generated with `cargo metadata --locked --format-version 1` using Rust
1.95.0. All 576 packages resolve through the crates.io registry;
none have a missing license expression. Rebuild the inventory when the lockfile
changes and review the packages included in each distributed target.

Bevy's `default_font` feature embeds a subset of Fira Mono in the application.
[Bevy credits](https://github.com/bevyengine/bevy/blob/v0.19.1/CREDITS.md)
attribute FiraMono to The Mozilla Foundation and Telefónica S.A. under the
SIL Open Font License 1.1. Distributions containing the embedded font must
carry its copyright and license notice. The font is not a separately added
project asset. Other locked packages include MIT, Apache-2.0, BSD, ISC, Zlib,
Unicode-3.0, and other permissive license expressions; distribute applicable
copyright and license notices with release artifacts. T14 performs the
release-specific notice review.

The local web page in `web/index.html` and all Rust source in this milestone
are original project files under the repository MIT License. The web packaging
uses [wasm-bindgen CLI 0.2.128](https://crates.io/crates/wasm-bindgen-cli/0.2.128)
as a development tool, matching the `wasm-bindgen` library in Cargo.lock;
the CLI is MIT OR Apache-2.0 licensed.

The component pixel art (`crates/app/src/sprites/`) is drawn in code and was
created for this project; it is covered by the repository MIT License. No
image files, fonts, or other third-party assets were added for it. The golden
references in `docs/design/sprites/` are generated from that code.
