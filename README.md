<!-- [![Build Status (Travis-CI)](https://travis-ci.com/OpenGM8/GM8Emulator.svg?branch=master)](https://travis-ci.com/OpenGM8/GM8Emulator) -->
[![Build Status (AppVeyor)](https://ci.appveyor.com/api/projects/status/5kad3dbn2q1jqs5i?svg=true)](https://ci.appveyor.com/project/viri/gm8emulator)
[![License: GPL v2](https://img.shields.io/badge/License-GPL%20v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)
[![Discord](https://discordapp.com/api/guilds/730417804368412686/widget.png?style=shield)](http://gmemu.com/discord)

**GM8Emulator** is a modern, open-source rewrite of the proprietary GameMaker 8 runner. It's being worked on almost every day! We’re constantly adding new features and updating the code.

Please remember that ___this emulator is a work in progress___ and is unreleased. There are MANY things we plan on adding, and the interface you are currently viewing will also change as we add in those features.

Until this emulator is officially released, please note that your saves may not work in updated versions, but see the "runtime errors" section below on how to work around this.

Also please keep in mind that we’re all volunteers making this passion project because we love fangames and want to contribute to the community  


# Starting your project!

***Note: all command steps will be streamlined in a future release***

This project uses Rust. Visit this link to install it: https://www.rust-lang.org/tools/install

After installing, if you cannot access Rust commands like "rustup" or "rustc", make sure `%USERPROFILE%\.cargo\bin` is in your PATH by typing the following into your CLI:

`Set path=”%USERPROFILE%\.cargo\bin”`

After cloning the GM8Emulator repository, set up Rust in your CLI by typing the following:

- `rustup self update`
- `rustup update`
- `rustup install nightly`
- `rustup default nightly`

Once that is set up, build the program in your CLI by typing `cargo build --release`, then navigate to the `/target/release` folder to start making and running your TAS! Note that you can run the build commmand again from this folder if you need to.

### TASing

Run the following commands in your CLI in `target/release`, replacing anything in the `<angle brackets>` as needed:

- Run a game in the emulator: `gm8emulator.exe <game.exe_location>`
- Start a TAS: `control-panel.exe <game.exe_location> -n <project_name>`
- Run a TAS: `gm8emulator.exe <game.exe_location> -f <save#.bin_location>`
  - Note: running a TAS will generate a <save#.gmtas> file

# Load / Runtime Errors

**Loading a game gives "Runtime error: invalid u8 while decoding bool..."**

> This means that the `save#.bin` format in your current project directory has changed and is not compatible with the current emulator's version. Please remember this emulator is a WIP and until it's officially released your saves may not work in updated versions.
> 
> To solve this issue, when you make a TAS copy down the git commit hash you're running by typing `git log -1`. If you ever update type `git reset <hash>` to return to that version whenever running that file. You can also get the hash in the Github repo.
> 
> (when this program is officially released, we will most likely have a file converter to ensure backwards compatibility)


**Loading a game gives "failed to load 'filename' - unknown format, could not identify file"**

> GM8Emulator works on GM8 games. It’s possible the game you are trying to load is a GMStudio game, which it does not support.


**Loading a game or during a game, I got "thread 'main panicked at...'**
**"'not implemented: Called unimplemented kernel function..." or**
**"'not yet implemented', gm8emulator\src\gml\runtime.rs..."**

> Your game tried to access functionality that isn’t supported in the emulator yet.


**Loading a game gives "Thread 'main' panicked at 'dll-bridge.exe could not be found."**

> Many games use 32-bit DLL files for things like audio. These cannot be directly called from 64-bit programs, so the workaround is to use a 32-bit bridge executable. Unfortunately Cargo doesn’t let us do this cleanly yet so it requires a little extra setup.
> 
> Run `rustup target add i686-pc-windows-msvc` to install the 32-bit build target, then run `cargo build --release` from the "dll-bridge" directory. This will place the dll-bridge.exe in "target/i686-pc-windows-msvc/release". You will need to move this into the folder with the emulator executable for it to work. After that, you should be sorted.

### Audio/Visual

**Why don’t I hear any sound?**

> The emulator doesn’t support the built-in audio engine yet. Some games, which use DLL files for audio, should work, however.

### Gameplay

**When I first load the game via the control panel, it doesn’t start at the title screen, or the usual starting location.**

> If your game starts at, say, the first screen after you usually select a difficulty, or anywhere else that’s unusual, check in your game's folder for a temp file. Once you delete and restart the control panel it should work normally.

---

### About GameMaker 8
GameMaker is an IDE for creating Windows games, developed by YoYo Games. *GameMaker 8* (GM8) was the last of the numbered releases of GameMaker, released on December 22nd 2009 (surpassing *GameMaker 7*) and succeeded by *GameMaker: Studio* in 2011. Due to the huge behavioral differences between "Numbered GameMaker" and *GameMaker: Studio*, as well as Studio's lack of backward-compatibility, GM8 is still widely used, with thousands of games to its name. One of GameMaker's strengths as a game engine is its ability to compile an entire project into a single executable. No external dependencies or installers, just compile, send the .exe file to your friend and they will be able to play your game. This is achieved by having the target executable act as a phase file for the entire collection of assets required to run the game. In other words, that .exe file contains not only the game engine code, but all of the objects, GML code, sprites, room layouts etc. required for the game logic. This behaviour was made optional in *Studio*, giving the creator a choice between standalone executable or .msi installer (although it extracts the contents to a temporary folder instead of having the executable contain all the assets), but in GM8 this is the only build option with Windows being the only build target.

The goal of this project is to create a program which will be able to parse GM8 .exe files and play the game contained within. It should mimic the behaviour of GameMaker 8's engine as closely as possible, down to the sub-frame. Strictly speaking, *emulator* is not a correct term. In computing, an emulator is a piece of software on a computer system which emulates the behaviour of a different computer system. We aren't emulating any computer system, just the GM8 engine. A more accurate term would be *sourceport* but emulator sounds cooler.

## Contributing
This project has only been worked on by a few people so far in their little free time, contributions are very welcome - however we'd encourage getting in touch beforehand.

## Credits
- [Adamcake](https://github.com/Adamcake)'s absurd amount of runtime research.
- [DatZach](https://github.com/DatZach)'s [decompiler](https://github.com/WastedMeerkat/gm81decompiler) for "documenting" the loading sequence.
- [Jabberwock-RU](https://github.com/Jabberwock-RU) for the new project (& organization) icon.
