# Capsense

Repurpose the CapsLock key for switching input methods on Windows.

## Why?

On Windows, the default shortcuts for switching input languages (like `Shift` and `Win + Space`) can often be confusing
or inconsistent across different IME (Input Method Editor) setups.

MacOS solves this by utilizing the often-underused `CapsLock` key as a dedicated toggle for input sources. **Capsense**
brings this behavior to Windows, allowing you to use a quick tap of the `CapsLock` key to trigger your preferred input
switching shortcut (defaulting to `Win + Space`).

## Features

- Use a short tap of `CapsLock` to switch input methods.
- Long press (or toggling via other means) still allows you to use CapsLock for its original purpose if needed (
  depending on your configuration).
- Runs efficiently in the background with minimal resource usage.
- Easily change the tap threshold and the shortcut triggered.

## Usage

Simply run the executable to start monitoring `CapsLock` events.

### Arguments

The program supports the following command-line arguments:

- `--startup <enable|disable>`: Enable or disable the program starting automatically with Windows.
- `--stop`: Stop running instance of Capsense.
- `--reload`: Reload the configuration from `config.toml`.

## Configuration

On first run, a `config.toml` file will be created in the same directory. You can customize the following:

- `tap_threshold_ms`: The maximum duration (in milliseconds) for a `CapsLock` press to be considered a "tap".
- `tap_shortcut`: The shortcut to trigger (e.g., `["LWIN", "SPACE"]`). Supported keys are:
    - `LWIN` (or `WIN`)
    - `SPACE`
    - `LCONTROL` (or `CTRL`)
    - `LSHIFT` (or `SHIFT`)
    - `LMENU` (or `ALT`)
    - `CAPSLOCK`

## License

```
Capsense is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

Capsense is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.
```