# Local 0.1.1 patch

`read` reads tar entries, YAML and image bytes in memory. Upstream unpacks into
`TempDir`, which calls `std::env::temp_dir()` and panics on browser WASM.
The `.ppd` format, image decoding, public API and native filesystem save remain.
WASM filesystem save returns `Unsupported` rather than attempting a temp directory.

Tests read the actual repository hero archive, verify every decoded image, read an
upstream-style `./` archive, report missing image errors, and succeed with an unusable
TMPDIR in an isolated native subprocess.
