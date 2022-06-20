trails of corroded steel
========================

a rusty toolbox for handling data from the trails of cold steel games.

it's a work in progress.

overview
--------

This project is split up into several crates (some libraries, some binaries) for technical limitations in rust.
Here's a short overview:

- `tocs` is the core library.
  It currently supports serialization and deserialization of tbl files for Cold Steel 1/2/3.
  Support for "structured parsing" (instead of having tbl entry data be an opaque blob) is very limited atm, but it will hopefully get better as I learn more about what everything does, which will hopefully happen as I build tools that allow me to learn it more quickly.
  If you want to programmatically modify tbl files in Rust, this is what you want.
- `tocs_internal` contains support libraries used internally by `tocs`.
  You should not use them directly, they are only separate due to a technical limitation in rust.
- `sqlite_conv` is a converter from tbls to a sqlite database.
  Currently only CS3 and tbl->sql is supported so it can be used for analysis of this game's tbls.
  I want to add support for other games and for sql->tbl.
  Once this is done, it should be much easier to edit tbl files.
- `try_parse` is a test program that attempts to parse all game data and shows errors.
  It's there for finding instances where the game data doesn't follow the rules we've reverse engineered.
  This can happen if the files are broken (it's more likely than you think), our understanding is incomplete, and/or because our parser has a bug.

related/alternative projects
----------------------------

- https://github.com/nnguyen259/ColdSteel3Tools
- https://github.com/nnguyen259/SenSchema
- https://github.com/Ouroboros/Falcom
- https://github.com/TwnKey/SenScriptsDecompiler
