# sued-rs

[![crates.io](https://img.shields.io/crates/v/sued-rs.svg)](https://crates.io/crates/sued-rs)
[![CI](https://github.com/Danilo-Guedes/sued-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/Danilo-Guedes/sued-rs/actions/workflows/ci.yml)
[![license](https://img.shields.io/crates/l/sued-rs.svg)](#license)
[![MSRV](https://img.shields.io/crates/msrv/sued-rs.svg)](https://www.rust-lang.org/tools/install)

A horror-themed terminal recreation of **SueD** — the 2000s Brazilian prank
oracle (_Sua Última Esperança Divina_, and "Deus" spelled backwards) — rebuilt
in Rust with [ratatui](https://ratatui.rs).

Light a candle. Turn off the lights. Ask it something you actually want to know.

![The invocation screen](https://raw.githubusercontent.com/Danilo-Guedes/sued-rs/main/docs/screenshots/intro_sangue.png)

![The oracle waiting for a question](https://raw.githubusercontent.com/Danilo-Guedes/sued-rs/main/docs/screenshots/ask_sangue.png)

<sub>Three themes ship with it — this is the same screen in **Âmbar**:</sub>

![The same screen in the Ambar theme](https://raw.githubusercontent.com/Danilo-Guedes/sued-rs/main/docs/screenshots/ask_ambar.png)

---

## What it is

SueD is a piece of stage magic wearing the costume of a program. Your victim
types a question into a black terminal, a demon watches them from the dark, and
the oracle answers — specifically, personally, and far too accurately.

**You are the oracle.** SueD gives you a way to answer in secret, while the
screen shows your victim exactly what they expect to see. Everything else —
the flickering demon, the incantations, the thunder, the laughter in the next
room — is theatre, and the theatre is the point.

It does **not** use AI, and it does **not** touch the network. The oracle is the
person at the keyboard.

## 🕯 The part you have to find

**The method is not written down here, and that is deliberate** — this page is
public, and your victim can read it too.

But it is not a secret from _you_. The program carries its own operator's manual:
the full trick, the timing, and how to perform it convincingly. It will not
appear on any screen the victim can see, so poke around the command line until
you find it. It is not hidden well. It is only hidden from the right person.

## Requirements

**A terminal at least 94×40.** Below that SueD shows a resize notice instead of
running — the illusion depends on nothing being clipped, and a truncated oracle
is not a frightening one. **132×41 is comfortable** and is the size the whole
thing was designed at.

Rust **1.88** or newer (2024 edition — the crate uses let-chains).

## Install

### Without Rust — a prebuilt binary

macOS, Linux and Windows builds are attached to every
[release](https://github.com/Danilo-Guedes/sued-rs/releases). Nothing to compile:

```sh
# macOS and Linux
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/Danilo-Guedes/sued-rs/releases/latest/download/sued-rs-installer.sh | sh
```

```powershell
# Windows
powershell -ExecutionPolicy Bypass -c "irm https://github.com/Danilo-Guedes/sued-rs/releases/latest/download/sued-rs-installer.ps1 | iex"
```

Or download a `.tar.xz` / `.zip` from the releases page and put the binary on your
`PATH`. If you have [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall),
`cargo binstall sued-rs` works too.

**Linux: the prebuilt binary needs ALSA's runtime library.** It is on essentially
every desktop, but a minimal server or container may not have it — there the
binary will not start at all:

```
error while loading shared libraries: libasound.so.2: cannot open shared object file
```

That is this, and nothing in that message says so. `sudo apt install libasound2`
(the plain package, **not** `-dev` — that one is only for building) fixes it.

### With Rust — from crates.io

**Linux: install ALSA's development headers first.** Sound is on by default, and
the build fails without them:

```sh
sudo apt install libasound2-dev    # Debian, Ubuntu
sudo dnf install alsa-lib-devel    # Fedora
```

macOS and Windows need nothing extra. Then, on any platform:

```sh
cargo install sued-rs
```

If you already tried and it stopped on `alsa-sys` with _"The system library
`alsa` … was not found"_ — that is this, and nothing in that error says so.
Install the headers above and run it again, or skip sound entirely with
[Building without audio](#building-without-audio).

### Known issue on Windows

Windows may refuse to run the compiled binary:

```
An Application Control policy has blocked this file. (os error 4551)
```

That is **Smart App Control**, not a permissions problem — running as
administrator will not help, and it has no per-file exclusion list. It blocks
executables it does not yet trust, and a freshly compiled one is always
unsigned. It is intermittent: the same binary is often allowed on a later
attempt.

If it persists, either run it under **WSL** (which runs the Linux build, where
Smart App Control does not apply — follow the Linux instructions above), or turn
Smart App Control off under Windows Security → App & browser control.

## Running

```sh
sued-rs                       # summon it
sued-rs --no-sound            # summon it quietly
sued-rs --config <PATH>       # use a specific config file
sued-rs --version             # print the version
sued-rs --help                # the flags, including one worth finding
```

Navigate with the arrow keys, `Enter` to choose, `Esc` to go back, `Ctrl+C` to
leave in a hurry.

## Configuration

Settings live in `~/.config/sued-rs/sued.config.json`. The file is optional and
so is every key in it — anything missing falls back to a default. You can change
all of it from the **Configuration** screen inside the app, and the changes apply
immediately.

| Setting        | Options                                                                      |
| -------------- | ---------------------------------------------------------------------------- |
| **Theme**      | `Sangue` (blood red, default) · `Ambar` (amber) · `Fosforo` (phosphor green) |
| **Language**   | English (default) · Português (BR) · Español                                 |
| **Animations** | on / off — turns off flicker, shake and the typewriter reveal                |
| **Volume**     | 0–100                                                                        |

## Building without audio

Audio is a Cargo feature, on by default. Turning it off removes the dependency
entirely, so no ALSA headers are needed to build:

```sh
cargo install sued-rs --no-default-features
```

`--no-sound` differs: it builds the audio support and stays quiet at runtime.
The feature flag is for machines that cannot build it at all.

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

**The bundled audio is not covered by those licences.** Eight sound files ship
inside the binary, each under its own terms — four of them CC-BY, where credit
is a condition of redistribution rather than a courtesy. The authoritative,
per-file list lives in [NOTICE](NOTICE) and travels with every copy of the
crate.

## A note on the original

SueD was a Brazilian internet classic of the 2000s, passed hand to hand on
diskettes and MSN. The trick underneath it is much older than the software, and
much older than computers — a piece of parlour magic that predates electricity,
briefly wearing a floppy disk as a disguise.

`sued-rs` is an homage, not a port: no code or assets from the original were
used, and nothing here was reverse-engineered. It is the same joke, told again, in a
language that did not exist when the joke was new.

**This project is unofficial.** It is not affiliated with, endorsed by, or
connected to the original SueD or the people who made it. It is a recreation
and a tribute, and it does not claim to be the original. If anyone with a
legitimate claim to the name would rather it were called something else, that
is a conversation worth having.

**And it is not occultism.** The demon, the incantations and the séance are
stage dressing, borrowed from a prank program of the 2000s that already looked
like this. `sued-rs` takes no position on anyone's religion or beliefs, and is
not meant to mock them.
