# Akari CLI

The Akari command-line tool creates, builds, and bundles applications built
with the Akari Rust GUI toolkit.

```sh
# After the first crates.io release:
cargo install akari-cli
akari new "Daily Notes" --app-id com.akari.dev
```

Generated projects include Rust sources and native Android/Kotlin and
iOS/Swift host scaffolding. Their Akari toolkit dependency comes from the
Akari Git repository; the toolkit crates themselves are not published to
crates.io.

Use `akari run` to launch a desktop app. Use `akari run --target android` to
install and launch the Android app on a connected device with USB debugging.

See the [Akari repository](https://github.com/metantesan/akari) for examples,
development setup, and platform status.
