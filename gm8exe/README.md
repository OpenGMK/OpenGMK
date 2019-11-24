# gm8exe

Library used for reading & writing executables created with GameMaker 8 into data structures.

Originally created for use in [GM8Emulator](https://github.com/OpenGM8/GM8Emulator),
most of the commits, diffs, etc are in that repository. Now moved to the decompiler
as it does the job of a decompiler more than an emulator (or runtime rewrite, rather).

## Documentation & Usage
The documentation is a best-effort and is not complete, you will probably need to read the source if you want to use this.

Not actually hosted anywhere, build it yourself with `cargo doc`. A good starting point is `reader::from_exe`.
