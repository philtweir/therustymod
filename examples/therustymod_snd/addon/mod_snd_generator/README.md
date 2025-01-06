# mod_snd_generator

**Assumes `x86` 64-bit Linux with GCC-compiled TDM.**

ZIP this as a PK4 and put it in the game directory,
beside `libtherustymod_snd.so` built from the directory
above and renamed to `mod_snd_generator_x86_64.so`.

### Instructions

1. Make this directory the current directory
1. `zip -r mod_snd_generator.pk4 *`
1. Go to `libtherustymod_snd` directory
1. `cargo build --release`
1. Copy `target/release/libtherustymod_snd.so` and `mod_snd_generator.pk4` to the game directory
1. Start The Dark Mod
1. Start a game
