trails of corroded steel
========================

a rusty toolbox for handling data from the trails of cold steel games.

for an introduction to modding trails games, see https://github.com/Trails-Research-Group

overview
--------

This project is split up into several crates (some libraries, some binaries).
Here's a short overview:

- `tocs` is the core library.
  It currently supports:

  - serialization and deserialization of tbl files for all Cold Steel games including Reverie.
  - packing/unpacking .pkg and .pka asset files
  - decoding BC7 textures (preliminary, barely tested)
  - serialization and deserialization of script files for Trails into Reverie (preliminary)

  If you want to programmatically modify tbl or pkg/pka files in Rust, this is what you want.

- `tbled` is a GUI tbl editor.
  For more information, see [its own README](tbled/README.md).

- `pkgtool` is a CLI program for packing/unpacking .pkg and .pka package files.
  For more information, see [its own README](pkgtool/README.md).

- `misc` contains miscellaneous functionality that isn't specific to the games.

- `sqlite_conv` is a CLI program to convert from tbls to a sqlite database.
  I use this for datamining and reverse engineering.
  The table schema is not going to be stable across versions, and backwards compatability is NOT a goal since it would be too much effort.
  As I learn more about the data, columns will be added/removed/renamed.

- `try_parse` is a CLI program that attempts to parse all game data and prints any errors.
  It's there for finding instances where the game data doesn't follow the rules we've reverse engineered.
  This can happen if the files are broken (it's more likely than you think), our understanding is incomplete, and/or because there's a bug in the code.

- `try_fix` is a CLI program that attempts to fix game data.
  It is not "intelligent" and all fixes are hardcoded.
  Only very basic formatting errors are fixed (so far it's only incorrect length bytes in tbls).
  This can be used to bring the files into a form where they can be more easily parsed with other software.

license
-------

¯\\\_(ツ)_/¯
