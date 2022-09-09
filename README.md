trails of corroded steel
========================

a rusty toolbox for handling data from the trails of cold steel games.

it's a work in progress.

overview
--------

This project is split up into several crates (some libraries, some binaries) due to technical limitations in rust.
Here's a short overview:

- `tocs` is the core library.
  It currently supports serialization and deserialization of tbl files for Cold Steel 1/2/3.
  Support for "structured parsing" (instead of having tbl entry data be an opaque blob) is very limited atm, but it will hopefully get better as I learn more about what everything does, which will hopefully happen as I build tools that allow me to learn it more quickly.
  If you want to programmatically modify tbl files in Rust, this is what you want.

- `tocs_internal` contains support libraries used internally by `tocs`.
  You should not use them directly.

- `sqlite_conv` is a CLI program to convert from tbls to a sqlite database.
  This sqlite database can be edited, e.g. with [sqlitebrowser](https://sqlitebrowser.org/) and converted back to tbl files.
  WARNING:
  The table schema is not going to be stable across versions, and backwards compatability is NOT a goal since it would be too much effort.
  As I learn more about the data, columns will be added/removed/renamed.
  If you created a database using some version of the program, it will likely be impossible to turn back to tbl using any other version.
  If you want to upgrade and need to persist the data from the database, first convert it to tbl files, then convert the tbls back to a new database after the upgrade.

- `tbled` is a GUI tbl editor.
  For more information, see [its own README](tbled/README.md).

- `try_parse` is a CLI program that attempts to parse all game data and prints any errors.
  It's there for finding instances where the game data doesn't follow the rules we've reverse engineered.
  This can happen if the files are broken (it's more likely than you think), our understanding is incomplete, and/or because there's a bug in the code.

- `try_fix` is a CLI program that attempts to fix game data.
  It is not "intelligent" and all fixes are hardcoded.
  Only very basic formatting errors are fixed (so far it's only incorrect length bytes in tbls).
  This can be used to bring the files into a form where they can be more easily parsed with other software.

related & alternative projects
------------------------------

- https://github.com/Trails-Research-Group
- https://github.com/nnguyen259/ColdSteel3Tools
- https://github.com/nnguyen259/SenSchema
- https://github.com/Ouroboros/Falcom
- https://github.com/TwnKey/SenScriptsDecompiler

license
-------

¯\\\_(ツ)_/¯
