tbled
=====

`tbled` is an editor for `.tbl` files for the Trails of Cold Steel games.
It is quite limited and obtuse, but I hope it is better than viewing or editing the files in a general purpose hex editor.

I try very hard to make sure this tool doesn't silently corrupt data.
However, there are two important possibilities for why this still may happen:
1\) I make mistakes sometimes.
2\) The tool does not necessarily preserve the order of entries within a .tbl file; while I have never noticed a problem with this I can't exclude that it might become one.
Therefore it is highly recommended to BACK UP ALL DATA FREQUENTLY.

`tbled` currently supports the following versions of the games:

- Trails of Cold Steel (XSeed PC version, english language only)
- Trails of Cold Steel II (XSeed PC version, english language only)
- Trails of Cold Steel III (NISA PC version)
- Trails of Cold Steel IV (NISA Switch version)

The following versions are untested but likely work:

- Trails of Cold Steel III (NISA Switch version)
- Trails of Cold Steel IV (NISA PC version)

The following versions are explicitely not supported at this time:
- Trails of Cold Steel (XSeed PC version, Japanese language)
- Trails of Cold Steel II (XSeed PC version, Japanese language)

Other versions you'll have to test yourself.

Usage
-----

To load a file, select one of the options under "Load" and select a `.tbl` file.
These files are located in the `data\text` folders relative to the installation path.
Make sure to choose the correct game version, or loading the data will likely fail.
If loading was successful, choose an entry type on the right to see all entries of that type in the loaded file.

Editing works via the edit field:

- Clicking on a cell in the table selects that cell and deselects all other cells.
- Double clicking a cell copies the content of that cell into the edit field.
- To edit the text in the edit field, make sure that it is active (e.g. by clicking on it).
- Once you're done editing, press Enter to copy the content of the edit field into the currently selected cell.

If you want to save your changes, select "Save" to overwrite the current file or "Save as" to write to a different file.

![A screenshot of `tbled` in action. Some UI elements are labeled](screenshot.png)

You can also use the CSV import/export function for editing:

- Load a .tbl, export a table (all entries with a given header) to CSV
- Edit the CSV in your favourite spreadsheet editor
- Import the updated CSV
- Save the updated .tbl file

Warnings
--------

The tool does certain checks to detect discrepancies in the data it loads.
In some cases, these checks can be too aggressive and reject data which would work perfectly well when loaded by the game.
Files with known schemas can be read under certain circumstances even when these checks fail.
For those cases, a list of warnings will be available under the "Warnings" menu option.
If there are warnings, it is your responsibility to check every warning and decide whether or not to fix the detected problem, as well as how to fix it.

If there are warnings for files that have previously been written with this tool, that's a bug.

Releases (downloads & changelogs)
---------------------------------

### [v0.4 (in development)]

- cleanup of code used to read/write and related code
- CSV import

### [v0.3.1](https://git.sr.ht/~quf/tocs/refs/download/tbled-v0.3.1/tbled-v0.3.1.exe)

- Fixed a bug that prevented saving files

### [v0.3](https://git.sr.ht/~quf/tocs/refs/download/tbled-v0.3/tbled-v0.3.exe):

- schemas for CS4
- improved handling of slightly-inconsistent files
- improved warning reporting

### [v0.2](https://git.sr.ht/~quf/tocs/refs/download/tbled-v0.2/tbled-v0.2.exe):

- improved UI
- improved performance
- support for saving to the current file (overwriting)
- added schemas for CS1 & CS2
- added a few schemas for CS3
- CSV export
- allow duplicating & removing entries

### [v0.1](https://git.sr.ht/~quf/tocs/refs/download/tbled-v0.1/tbled.exe):

- initial public version
