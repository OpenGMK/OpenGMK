[![Build Status (AppVeyor)](https://ci.appveyor.com/api/projects/status/ncr1f6vp2r76iah2?svg=true)](https://ci.appveyor.com/project/notviri/gm8emu)
[![Build Status (Travis)](https://travis-ci.com/notviri/gm8emu.svg?token=j2qsh1B3n1yUgL7VWJd5&branch=master)](https://travis-ci.com/notviri/gm8emu)
[![License: GPL v2](https://img.shields.io/badge/License-GPL%20v2-blue.svg)](https://www.gnu.org/licenses/old-licenses/gpl-2.0.en.html)

# gm8emu
A modern, open-source rewrite of the proprietary GameMaker 8 runner.

## About GameMaker
GameMaker is an IDE for creating Windows games, developed by YoYo Games. *GameMaker 8* (GM8) was the last of the numbered releases of GameMaker, released on December 22nd 2009 (surpassing *GameMaker 7*) and succeeded by *GameMaker: Studio* in 2011. Due to the huge behavioral differences between "Numbered GameMaker" and *GameMaker: Studio*, as well as Studio's lack of backward-compatibility, GM8 is still widely used, with thousands of games to its name. One of GameMaker's strengths as a game engine is its ability to compile an entire project into a single executable. No external dependencies or installers, just compile, send the .exe file to your friend and they will be able to play your game. This is achieved by having the exe contain a phase file for the entire collection of assets required to run the game. In other words, that game.exe file contains not only the game engine, but also the entire game. This behaviour was made optional GameMaker: Studio, giving the creator a choice between standalone executable or .msi installer (although it extracts the contents to a temporary folder instead of having the executable contain all the assets), but in GM8 this is the only build option.

The goal of this project is to create a program which will be able to parse GM8 .exe files and play the game contained within. It should mimic the behaviour of GameMaker 8's engine as closely as possible, down to the sub-frame.

## Contributing
This project has only been worked on by two people so far in their little free time, contributions are very welcome - however we'd encourage getting in touch beforehand.

## Credits
- [Adamcake](https://github.com/Adamcake)'s absurd amount of runtime [documentation](https://github.com/Adamcake/GM8Emulator/blob/master/notes.txt).
- [DatZach](https://github.com/DatZach)'s [decompiler](https://github.com/WastedMeerkat/gm81decompiler) for "documenting" the loading sequence.
