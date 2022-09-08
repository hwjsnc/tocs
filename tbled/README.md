tbled
=====

`tbled` is an editor for `.tbl` files for the Trails of Cold Steel games.
It is quite limited and obtuse, but I hope it is better than viewing or editing the files in a general purpose hex editor.

I try very hard to make sure this tool doesn't silently corrupt data.
However, there are two important possibilities for why this still may happen:
1\) I make mistakes sometimes.
2\) The tool does not necessarily preserve the order of entries within a .tbl file; while I have never noticed a problem with this I can't exclude that it might become one.
Therefore it is highly recommended to BACK UP ALL DATA EARLY AND OFTEN.

`tbled` currently supports the following versions of the games:

- Trails of Cold Steel (XSeed PC version, english language only)
- Trails of Cold Steel II (XSeed PC version, english language only)
- Trails of Cold Steel III (NISA PC version)

The tool does certain checks to detect discrepancies in the data it loads.
In some cases, these checks can be too aggressive and reject data which would work perfectly well when loaded by the game.
This can happen with a few of the original game files, or with files that have been manually hex edited.
If this happens, you can override a lot of the optional checks by running tbled from the terminal with the option `--nonstrict` (this option is intentionally hidden in the GUI).
If you do this, it is your responsibility to check every warning and decide whether or not to fix the detected problem, as well as how to fix it.
Failing to do this could silently corrupt the data.

Usage
-----

To load a file, select one of the options under "Load" and select a `.tbl` file.
These files are located in the `data\text` folders relative to the installation path.
Make sure to choose the correct game version, or loading the data will likely fail.
If loading was successful, choose an entry type on the right to see all entries of that type in the loaded file.

Editing works via the edit field.
Clicking on a cell in the table selects that cell and deselects all other cells.
Double clicking a cell copies the content of that cell into the edit field.
To edit the text in the edit field, make sure that it is active (e.g. by clicking on it).
Once you're done editing, press Enter to copy the content of the edit field into the currently selected cell.

If you want to save your changes, select "Save" to overwrite the current file or "Select & save file" to write to a different file.

![A screenshot of `tbled` in action. Some UI elements are labeled](screenshot.png)

Releases
--------

### v0.2 (in development):

- improved UI
- improved performance
- support for saving to the current file (overwriting)
- added schemas for CS1 & CS2
- added a few schemas for CS3
- CSV export
- allow duplicating & removing entries

### [v0.1](https://git.sr.ht/~quf/tocs/refs/download/tbled-v0.1/tbled.exe):

- initial public version
