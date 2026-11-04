# Framework fallback fonts

Akari bundles Noto Sans regular and Noto Sans Arabic regular as last-resort
fallback faces. Platform providers load the system UI font first and retain
these faces for missing glyphs or systems where the selected font is not
readable by the application.

The fonts are from the [Noto fonts project](https://github.com/notofonts/latin-greek-cyrillic)
and are Copyright 2013 Google LLC under the SIL Open Font License 1.1. The
license is included in this directory and in the workspace-root `LICENSE`.
