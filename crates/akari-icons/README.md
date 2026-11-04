# Akari Lucide icons

`icons/` contains the upstream Lucide SVG source at the revision recorded in
`LUCIDE_COMMIT`. The workspace-root `LICENSE` contains the upstream license
texts and attribution notices.

Cargo runs `build.rs` and the Handlebars template in `templates/` to generate
one named `Icon` value per SVG under `OUT_DIR`. Applications can use values such
as `akari::icons::lucide::arrow_up`.
Hyphens become underscores; a trailing underscore is added when an icon name
matches a Rust keyword. The generated `lucide::ICON_COUNT` reports the number
of vendored SVGs. Normal builds expose no global icon registry, so applications
can reference individual values and let unused icon data be removed by Rust's
linker.

The generator accepts Lucide `path`, `circle`, `ellipse`, `line`, `polyline`,
`polygon`, and `rect` elements. When updating the pack, replace `icons/` with
the SVG directory from a chosen upstream commit, update `LUCIDE_COMMIT` and
`LICENSE` from that same commit, then run `cargo test -p akari-renderer
every_generated_lucide_icon_flattens_to_drawable_segments` to validate the
generated geometry against Akari's renderer.
