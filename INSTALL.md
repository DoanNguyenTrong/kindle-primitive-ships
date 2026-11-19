# Install Primitive Ships

When available in Primitive Store, open the listing and choose Get. Open Primitive Ships from the launcher grid.

For a local USB test, extract the package for your Kindle, copy the `ships` and `documents` folders to user storage,
eject, and open Primitive Ships from Primitive Launcher. The app uses the launcher's fonts. The Done button returns you
to the launcher.

## Build a local package

Run from this directory with the shared build and tools checkouts next to it.

```sh
cargo zigbuild --offline --locked --release -p ships-kindle --target armv7-unknown-linux-musleabihf
sh scripts/stage.sh armhf
python3 ../doannt-kindle-build/scripts/package.py --id primitive-ships --dir ships \
  --version 0.1.0 --arch armhf --stage build/stage --license LICENSE \
  --docs-license LICENSE-DOCS --install INSTALL.md --out dist
```
