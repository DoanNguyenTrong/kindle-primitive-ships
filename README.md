# Primitive Ships

Set out four ships, then look for the opposing fleet on an eight-by-eight map. Easy chooses a random square, Standard pursues nearby hits, and Hard weighs possible ship positions. In two-player mode a handoff screen covers both maps between turns.

Run `cargo test --offline --workspace` and `cargo run --offline -p ships-desktop` with
`doannt-kindle-tools` and `doannt-kindle-launcher` beside this checkout. The desktop target
writes 1264 × 1680 grayscale previews in `build/previews/`.

See [installation instructions](INSTALL.md). The local `app.json` is a listing draft. A
public listing requires approved artwork and an asset from a published release.

Source is PolyForm Noncommercial 1.0.0. Documentation is CC BY-NC-SA 4.0.
