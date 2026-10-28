# 2.3.1
- Game folder auto-detect now also finds GOG (Galaxy and offline installers, via GOG's registry entries) and Epic Games Store installs, plus GOG/Epic installs made through the Heroic launcher on Linux
- Updated egui/eframe to 0.37 (still using the OpenGL renderer)
- Require json5 1.3.6 or newer, avoiding 1.3.4, which made loading large saves extremely slow

# 2.3.0
Action by @lin0287:
- Tech progress editor for `techProgress`: edit each tech's accumulated research and per-faction contributions, with faction names resolved
- Game folder setting: on first launch TISE asks for your Terra Invicta install folder (auto-detects Steam installs), remembers it in a config file, and reads base-game and enabled-mod templates from it
- Councilor traits editor: click traits to remove them, search the game trait list to add them, with warnings for unknown or duplicate names
- Groups panel is now grouped into collapsible categories (Factions & People, Earth, Space, Facilities, Missions & Events, AI Goals, Global System, Other) instead of one long flat list
- Project progress editor: new projects go into the first free valid slot (3, then 4/5 if the org/hab slot is unlocked, then 6+; slots 0-2 are ignored by the game), the picker only offers the faction's available projects, and cards warn about entries in slots below 3 or for already-finished projects
- Project progress editor: removed the misleading "Completed" checkbox, and each card shows the project's research cost from the game templates (repeatable projects scale as base cost x (1 + times finished)); repeatable projects can be added multiple times without warnings
- Game data: the one-time project list no longer drops projects when a mod override omits `oneTimeGlobally`
- Singleton groups (TIMetadataState, TITimeState, TIGlobalResearchState, TIGlobalValuesState) auto-select their only object and skip the Objects panel; other groups with a single object still show it

# 2.2.0
- In the Organizations group, new "Add to Faction" feature! Thanks @lin0287 for the contribution -- pull #24
- In-app update checker confirmed working!

# 2.1.3
- Testing "release" for in-app update checker -- pull #23

# 2.1.2
- Unreleased, CI smoke test for Cargo-git-tag version checker

# 2.1.1
- Add No Faction Filter for Organizations -- pull #21

# 2.1.0
- New faction/group filters! Thanks @lin0287 for the contribution -- issue #13
- Save comparison: structural diff between two saves with a noise-reduction ignore preset and a Compare Saves window -- issue #12
- List editor enhancements: add/remove/duplicate rows with per-row type selection -- issue #11
- CLI flags: `--version` and `--help` now exit without launching the GUI
- CI: headless GUI smoke test under xvfb, and fixed the Linux release artifact filename -- issue #17

# 2.0.0
- Complete rewrite in Rust!
  - Indirectly fixes issue #8 since we no longer use Poetry and Python
- New GUI with dark mode / light mode support!
- New search functionalities!
- New reference browser! Issue #3
- Undo/Redo support!
- Tests! Yes that also gets an exclamation point. Fixed line endings bug, issue #7

# 1.2.0
- Fix group list bug where duplicate names were not being shown. Thanks @Kazcade for reporting. Also added a sort by name or ID feature #4

# 1.1.1

- Fixed bug when saving references, "$type" attribute gets pruned

# 1.1.0

- Added support for compressed files -- GitHub issue #1
- Fixed bug when saving references, wasn't in original format
- Added support for going back and forward. New menu bar buttons, also bound to mouse fwd/back buttons -- GitHub issue #2

# 1.0.1

- First public release
