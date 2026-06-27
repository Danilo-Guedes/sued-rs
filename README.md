# sued-rs

A horror-themed terminal (TUI) recreation of **SueD** — the 2000s Brazilian prank
"oracle" (*Sua Última Esperança Divina* / "Deus ao contrário"), rebuilt in Rust.

> ⚠️ **Status: early / work in progress.** This is the M0 scaffold. Follow the
> milestones as they land.

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

## Build & run

```sh
cargo run            # build and run (no audio by default)
cargo test           # run the unit tests
```

Audio is an optional feature so the project builds without ALSA dev headers:

```sh
cargo run --features audio    # with sound (Linux needs: sudo apt install libasound2-dev)
```

## License

Dual-licensed under either **MIT** or **Apache-2.0**, at your option.
