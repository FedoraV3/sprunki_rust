# Sprunki, but in Rust

A tiny [Incredibox](https://www.incredibox.com/) / Sprunki-style beat toy built
with [macroquad](https://macroquad.rs/).

Every character owns one looping sample. Switch a character on and its sample
plays on a shared bar, so everything you stack stays in sync.

## Controls

| Input                     | Action                                  |
| ------------------------- | --------------------------------------- |
| Click a character         | Toggle them on or off                   |
| Character's key (`O` for Oren) | Toggle them on or off              |
| `Space`                   | Stop everyone                           |
| `Esc`                     | Quit                                    |

A character switched on while the beat is already running waits for the next
bar before joining, so nothing ever plays out of time.

## Running

```sh
cargo run --release
```

On Linux you need the ALSA development headers (`libasound2-dev` on Debian
and Ubuntu, `alsa-lib-devel` on Fedora).

## Adding a character

Drop a PNG into `src/Sprites/` and a WAV into `src/Sounds/`, then add an entry
to the `CHARACTERS` table at the top of `src/main.rs`:

```rust
CharacterAssets {
    name: "Sky",
    key: KeyCode::S,
    sprite: include_bytes!("Sprites/sky.png"),
    sample: include_bytes!("Sounds/sky.wav"),
},
```

Characters are laid out in a grid automatically. The bar length is taken from
the longest sample, so keep your loops the same length for the tightest result.

## Tests

```sh
cargo test
```
