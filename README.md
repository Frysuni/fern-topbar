# Topbar

A GTK panel for Wayland, with Niri integration.

## Configuration

Copy `config.example.json` to `~/.config/topbar/config.json` and edit it.
The file uses JSON (no comments or trailing commas). All supported options and
widgets are included in the example.

Choose another file with either spelling:

```sh
topbar --config /path/to/config.json
topbar -c /path/to/config.json
```

File selection priority:

1. `--config` / `-c`
2. `TOPBAR_CONFIG`
3. `$XDG_CONFIG_HOME/topbar/config.json`
4. `~/.config/topbar/config.json`

A missing default file uses built-in defaults. A missing explicitly selected file
is an error.

Changes are detected automatically, including saves that replace the file by
renaming a temporary file. After roughly 250–500 ms of stable contents, topbar
validates the configuration and restarts itself to apply all settings. Open menus
and in-memory notification history are reset by this restart. The selected file
continues to be watched. Removing the file leaves the current settings intact.
After a successful reload, a normal-priority notification confirms that the new
configuration was applied and closes after 5 seconds. Regular startup and
`validate` do not display this notification.

Invalid edits keep the running configuration and display a critical notification
for 10 seconds after delivery. Repeated errors replace that notification. An
unchanged invalid file is reported only once; saving a corrected file resumes
automatic reload. Startup errors are printed to stderr and, when a desktop
notification server is available, also shown as a 10-second critical notification.

## Validation

```sh
topbar validate
topbar validate --config /path/to/config.json
topbar -c /path/to/config.json validate
```

Validation checks JSON syntax, option names, types, scale limits, duplicate
widgets, and dependent options. It does not start GTK, show notifications, or
require a graphical session. Hardware/service availability and connected monitor
names are checked when the panel runs, not by this offline validation command.
If the default file is absent, the built-in defaults are validated.

Exit codes: `0` for a valid configuration, `1` for a configuration/read error,
`2` for incorrect command-line usage. Use `topbar --help` for command syntax.
