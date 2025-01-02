# mod_web_browser

**Assumes `x86` 64-bit Linux with GCC-compiled TDM.**

ZIP this as a PK4 and put it in the game directory,
beside `libtherustymod_web.so` built from the directory
above and renamed to `mod_web_browser_x86_64.so`, and
you should get a web browser on `localhost:9797`
that will start showing "init" when you load a game.

### Instructions

1. Make this directory the current directory
1. `zip -r mod_web_browser.pk4 *`
1. Go to `libtherustymod_web` directory
1. `cargo build --release`
1. Copy `target/release/libtherustymod_web.so` and `mod_web_browser.pk4` to the game directory
1. Start The Dark Mod
1. Load up `http://localhost:9797` in a browser and see a blank page
1. Start a game
1. Refresh the browser and see `init` on an otherwise blank page
