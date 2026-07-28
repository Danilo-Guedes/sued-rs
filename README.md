# sued-rs

A horror-themed terminal (TUI) recreation of **SueD** — the 2000s Brazilian prank
"oracle" (*Sua Última Esperança Divina* / "Deus ao contrário"), rebuilt in Rust.

> 🩸 **Status: playable.** The full prank runs end-to-end — a navigable 5-screen
> spooky TUI (intro · menu · question · info · about) over a unit-tested, I/O-free
> prank engine. Audio, terror effects and config are the milestones still landing.

## What it is

SueD is a piece of stage magic dressed up as software. The victim believes the
program magically answers any question they ask. In reality, the **operator
secretly types the answers** while pretending to type the question — a hidden-mode
toggle on the `;` key swaps real keystrokes into a hidden buffer and shows *decoy*
text on screen. The candles-in-the-dark, demonic presentation is all theater to
sell the illusion.

**Cultural note.** SueD is a Brazilian-internet classic from the 2000s. The
underlying trick — a fake fortune-teller where the operator secretly supplies the
answers — is far older than the software and is essentially pre-digital stage magic.
`sued-rs` is a faithful, modern, cross-platform homage. It does **not** use any AI
and does **not** connect to the network; the "oracle" is the person at the keyboard.

## What works now

- **The prank, end-to-end** — the hidden-mode (`;`) toggle, the decoy that "types
  itself," and the reveal.
- **The full spooky TUI** — five keyboard-navigable screens with a merged full-bleed
  red frame, demon ASCII art and the SUED banner (arrows · Enter · Esc · Ctrl+C).
- **A pure, tested core** — the trick logic lives in an I/O-free engine; 33 tests green.

Still landing: looping dread audio + a jump-scare sting, terror effects (a char-by-char
reveal, flicker, screen-shake), and config/CLI (themes, languages, `--no-sound`).

## Build & run

```sh
cargo run            # build and run (audio ON by default)
cargo test           # run the unit tests
cargo run -- --no-sound       # run silent
```

Audio is a Cargo feature, on by default. Turn it off at build time if you don't
want the ALSA dev headers as a dependency:

```sh
cargo run --no-default-features   # builds with no audio at all
```

On Linux an audio build needs `sudo apt install libasound2-dev`.

## License

Dual-licensed under either **MIT** or **Apache-2.0**, at your option.

This covers the **code**. The bundled audio is third-party and carries its own
terms, listed below.

## Audio credits

Four sound files ship with this crate, embedded into the binary at compile time.
Each carries its own licence. All were transcoded to Ogg Vorbis for the build;
any other change is noted per file.

**`assets/ambience.ogg`** — the looping dread bed
"Dark horror ambience" by **LukaCafuka** — [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/)
<https://freesound.org/people/LukaCafuka/sounds/758478/>

**`assets/laugh.ogg`** — the intermittent laughter
"Evil Laugh 1" by **prometheus_crr** — [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/)
<https://freesound.org/people/prometheus_crr/sounds/593305/>

**`assets/jump_scare.ogg`** — the reply sting
"Piano Scare" by **ERT3001** — [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/)
<https://freesound.org/people/ERT3001/sounds/723292/>

**`assets/thunder.ogg`** — the decoy-exhaustion warning
"rock_breaking" (from *Yo Frankie!*) by the **Blender Foundation**
[CC-BY 3.0](https://creativecommons.org/licenses/by/3.0/) —
<https://opengameart.org/content/rockbreaking>
*Modified: transcoded from FLAC to Ogg Vorbis.*

CC0 files require no attribution; it is given here as a courtesy. The CC-BY file
**does** require it, which is why its entry names the author, the licence and the
modification.
