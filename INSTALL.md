# Install Ships

After [setting up PrimitiveOS](https://github.com/PrimitiveOS/kindle-primitive-launcher/blob/main/docs/INSTALL.md#first-time-setup-over-usb), open **Store** from Primitive Launcher.
Select **Ships** under **Apps**, tap **Get**, then open it from the Launcher grid.

For a local USB test, extract the package for your Kindle, copy the `ships` and `documents` folders to user storage,
eject, and open Ships from Primitive Launcher. The app uses the launcher's fonts. The Done button returns you
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
