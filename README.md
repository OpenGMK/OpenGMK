[![License: GPL v2](https://img.shields.io/badge/License-GPL%20v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)
[![Discord](https://discordapp.com/api/guilds/730417804368412686/widget.png?style=shield)](http://gmemu.com/discord)

# OpenGMK

**OpenGMK** is a modern, open-source rewrite of the proprietary GameMaker Classic engines,
providing a full sourceport of the runner, a decompiler, and the ability to record TASes.
It's being worked on almost every day! We’re constantly adding new features and updating the code.
Please remember that ___this project is a work in progress___ and is unreleased.
Until there's an official release, please note that your saves may break in future releases.
See runtime errors section below on how to work around this.

## Building GM8Emulator / GM8Decompiler

This project is written in the Rust programming language. Download the toolchain manager directly from https://rustup.rs/ or a package manager of your choice.
After installing, make sure you're up to date and then download the nightly branch:

- `rustup self update`
- `rustup update`
- `rustup install nightly`

Clone the OpenGMK repository.
We use a few git submodules, so initialise them while cloning like so:

- `git clone --recurse-submodules https://github.com/OpenGMK/OpenGMK.git`

Once that's set up, `cd` to the repository and build the entire project
in the release profile so it's optimized (this will take a while):

- `cargo +nightly build --release`

The build artifacts will be located in `(repo folder)/target/release/`.
If you're on Windows 64-bit and would like to play games with GM8Emulator
that require 32-bit DLLs to function (such as *GMFMODSimple* or *supersound*)
you'll also need to build the WoW64 server, preferably in the release profile.
It requires the additional installation of the `i686-pc-windows-msvc` target with rustup.

- `rustup target add --toolchain=nightly i686-pc-windows-msvc`
- `cd gm8emulator-wow64`
- `cargo +nightly build --release`

The build artifacts for the WoW64 server will be in
`(repo folder)/gm8emulator-wow64/target/i686-pc-windows-msvc/release/`.
The binary should either be manually copied to the same folder as `gm8emulator` to work,
or the `OPENGMK_WOW64_BINARY` environment variable should be set
with the path to the binary.

A much easier alternative to this is building the project as 32-bit on Windows,
where the WoW64 server is not required and the DLL loading logic is bundled inside GM8Emulator.
It should be noted that cross-platform extension emulation is planned for the long-term future.

## Recording & Replaying TASes with GM8Emulator

- Play a game normally: `gm8emulator <game_exe_location>`
- Record a TAS: `gm8emulator <game_exe_location> -n <project_name>`
  - If this is a new project, it'll be created in `(working directory)/projects/project_name/`.
  - If this is an existing project, it'll resume it from that same path if it exists.
- Replay a TAS: `gm8emulator <game_exe_location> -f <save#{.bin,.gmtas}>`
  - A `save#.bin` file is generated for each savestate in record mode.
  - A `save#.gmtas` can be exported from record mode, which is inputs only.

*Note that all command-line steps will be streamlined in a future release.*

## Load / Runtime Errors

**Loading a game gives "failed to load 'filename' - unknown format, could not identify file"**

> OpenGMK is made to support **GameMaker Classic** games. It’s possible the game you are trying to load was actually made with the newer **GameMaker: Studio**,
which it does not have support for at the moment.
Whether it will in the future is unclear right now.

**Loading a game gives "invalid u8 while decoding bool" or "expected variant index" (or similar)**

> This means that the `save#.bin` format in your current project directory has changed
> and is incompatible with the current GM8Emulator.
> This is a byproduct of the emulator being actively developed and changing a lot.
> This will not be an issue when it's fully released.
>
> There are two ways to fix this:
>
> - You can export a `save#.gmtas` file, and then beg Adam to add the converter to the repo.
> - You can downgrade your local repository to the last version it worked on.
>   - View the latest commit in your cloned repo: `git log -1` (or look on GitHub)
>   - Make sure you're up to date with the remote repo: `git fetch --all`
>   - Seek to a specific commit: `git reset --hard <hash>`
>   - Rebuild the project as per instructions above.

**Loading a game or during a game "called unimplemented kernel function" or**
**"not yet implemented" (or similar)**

> Your game tried to access functionality that's yet to be implemented.
> The full GameMaker standard library is massive and there's a good bit left to cover.

## About GameMaker Classic
**GameMaker** is an engine for creating Windows games, developed by YoYo Games.
*GameMaker 8* ("GM8") was the last of the numbered releases of GameMaker,
released on December 22nd 2009 (surpassing *GameMaker 7*)
and succeeded by the vastly more popular *GameMaker: Studio* in 2011.
The pre-Studio versions are often referred to as *GameMaker Classic*.
Due to the huge behavioral differences, as well as *Studio*'s lack of backward-compatibility,
the classic engines are still very widely used, with thousands of games to their name.
One of GameMaker's original strengths as a game engine were
its ability to compile an entire project into a single executable.
No external dependencies or installers, just compile,
send the `.exe` file to your friend and they will be able to play your game.
This is achieved by having the target executable act as a phase file
for the entire collection of assets required to run the game.
In other words, the executable contains not only the game engine code,
but all of the objects, scripts, sprites, room layouts, everything required for the game logic.
This behaviour was made optional in *Studio*, giving the creator a choice between
a standalone executable or `.msi` installer, however the standalone builds
just extract the contents of the installer to a temporary folder when they're launched.

This project was originally started as
[**`GM8Emulator`**](https://github.com/Adamcake/Legacy-GM8Emulator),
a program that can load
*GameMaker Classic* games and accurately play the game within.
It should mimic the behaviour of the original engine as closely as possible,
down to the sub-frame and ideally implementation detail (if observable).
Strictly speaking, *emulator* was not a correct term.
In computing, an emulator is a piece of software on a computer system
which emulates the behaviour of a different computer system.
We aren't emulating any computer system, just the engine.
A more accurate term would be *sourceport* but emulator sounded cooler at the time.
The project required us to write a decompiler to get the assets,
and since we were already maintaining a fork of the 2013
[**`gm81decompiler`**](https://github.com/WastedMeerkat/gm81decompiler)
we made our own much faster version out of the new codebase,
[**`GM8Decompiler`**](https://github.com/OpenGMK/GM8Decompiler),
which was originally a separate repository but was eventually merged
into the unified repository known as **`OpenGMK`**
where all related (and future) projects will be hosted.
Releases of the decompiler are still available to download from the
[decompiler repository](https://github.com/OpenGMK/GM8Decompiler).

## Contributing

This project has only been worked on by a few people so far in their little free time.
Contributions are always welcome, although it's highly preferred to get in contact beforehand
to discuss details.

## Additional Credits
- [DatZach](https://github.com/DatZach)'s [decompiler](https://github.com/WastedMeerkat/gm81decompiler) for "documenting" the loading sequence.
- [Jabberwock-RU](https://github.com/Jabberwock-RU) for the new project (& organization) icon.
