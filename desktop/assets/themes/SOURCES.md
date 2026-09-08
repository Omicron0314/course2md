# Built-in palette sources

Retrieved 2026-09-08. `desktop/src/palettes.rs` contains the application mapping;
`theme.rs` derives semantic hover, selection, status and elevated colors from it.

| Family | Upstream | Mapping |
| --- | --- | --- |
| Paper / Ink | course2md | Original neutral light/dark pair with blue actions. |
| Nord / Nord Snow | [Nord palette](https://www.nordtheme.com/docs/colors-and-palettes), [source](https://github.com/nordtheme/nord/blob/develop/src/nord.scss) | Nord uses Polar Night backgrounds, Snow Storm text and Frost accent. Nord Snow is this application's light adaptation, with light surfaces and darker status foregrounds. |
| Tokyo Night / Day | [Tokyo Night](https://github.com/folke/tokyonight.nvim), [Night](https://github.com/folke/tokyonight.nvim/blob/main/extras/wezterm/tokyonight_night.toml), [Day](https://github.com/folke/tokyonight.nvim/blob/main/extras/wezterm/tokyonight_day.toml) | Retains the upstream backgrounds, blue accents, text and selection. Raised surfaces and some supporting/status text are adapted for a document interface. |
| Catppuccin | [Palette](https://github.com/catppuccin/palette/blob/main/palette.json) | Latte, Frappé, Macchiato and Mocha use their named text, base, mantle, crust, surface and blue colors. Dark canvas uses mantle and cards use base; Latte's raised surface and green/yellow foregrounds are adapted. |

Original licenses are in `licenses/` and included in distributed bundles. They
cover the palette source material; upstream projects do not endorse course2md.
Brand artwork has separate provenance in `../brands/SOURCES.md`.
