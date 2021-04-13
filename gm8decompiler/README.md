[![License: GPL v2](https://img.shields.io/badge/License-GPL%20v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)
[![Discord](https://discordapp.com/api/guilds/730417804368412686/widget.png?style=shield)](http://gmemu.com/discord)

# GM8Decompiler
An open-source decompiler for GameMaker 8.x executables.
Reverts any game back to .gmk or .gm81 format respectively.

## How it works
GameMaker 8 executables contain two sections:
the regular part which is virtualized by Windows, called the "runner",
and a phase file containing all the game's assets, called the "gamedata".
The gamedata contains all of the assets (sprites, rooms, GML code, etc...)
which were exported from the GMK file when the game was built.
When the game is run, it reads its gamedata section from disk and uses it to start the game.
Since all the assets can be read from the gamedata by anyone who has the file,
it is possible to revert it to its original project file.
That's what this tool does.

## Background
Originally, we created a fork of [WastedMeerkat's gm81decompiler](https://github.com/WastedMeerkat/gm81decompiler)
which, while an excellent resource, was very messy and had several deep-seated bugs.
For that reason and a few others, we eventually decided to create this project from scratch in Rust.
It's based on our new gm8exe library, originally created for emulation purposes.
This loader has been measured to be **over ten times faster** than the old one.
It's also safer, more thorough, and supports more games.
