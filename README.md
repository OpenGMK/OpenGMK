[![Build Status (Travis)](https://travis-ci.com/OpenGM8/GM8Decompiler.svg?branch=master)](https://travis-ci.com/OpenGM8/GM8Decompiler)

# GM8Decompiler
An open-source decompiler for GameMaker 8 executables. Reverts any GM8 game back to .gmk or .gm81 format.

## How it works
GameMaker 8 executables contain two sections: the regular part which is virtualized by Windows, called the "runner", and a phase file containing all the game's assets, called the "gamedata". The gamedata contains all of the assets (sprites, rooms, GML code, etc...) which were exported from the GMK file when the game was built. When the game is run, it reads its gamedata section from disk and uses it to start the game. Since all the assets can be read from the gamedata by anyone who has the file, it is possible to extract them all and create a GMK from them. That's what this does.

## Background
My old decompiler was based on a fork of [WastedMeerkat's gm81decompiler](https://github.com/WastedMeerkat/gm81decompiler) which, while an excellent resource, was very messy and had several deep-seated bugs. For that reason and a few others, we eventually decided to create this project from scratch in Rust. It's based on the gm8exe library, [which we now moved to this repo](./gm8exe/), originally created for emulation purposes. This loader has been measured to be over ten times faster than the old one. It's also safer, more thorough, and supports more games.

## Contact
For any enquiries contact gm8emulator@gmail.com
