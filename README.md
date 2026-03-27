<img height="200" alt="image" src="https://github.com/user-attachments/assets/c324645d-683c-4cc6-b744-60d8b6bbfc58" />

**English** | [`中文`](README_CN.md)

# Capsense

Repurpose the CapsLock key for switching input methods on Windows.

## Why?

On Windows, the default shortcuts for switching input languages (such as `Shift` and `Win + Space`)
can cause confusion about the current input method status.

MacOS solves this by utilizing the often-underused `CapsLock` key as a dedicated toggle for input sources. **Capsense**
brings this behavior to Windows, allowing you to use a quick tap of the `CapsLock` key to trigger IME switching, while
still retaining the ability to use `CapsLock` for its original purpose.

## Features

- Use a short tap of `CapsLock` to switch input methods.
- Long press to use CapsLock for its original purpose.
- Runs efficiently in the background with minimal resource usage.
- Easily change the tap threshold and the shortcut triggered.

## Usage

Simply run the executable to start monitoring `CapsLock` events.

If you double click the executable, there will be no window or console, but the program will be running in the
background.

We recommend setting your IMEs to disable the functionality of the `Shift` key for switching IME states to get the best
experience, as it can confuse you. Instead, you use `CapsLock` to switch keyboard layouts, Use `Win+Space` to switch the
primary IME of the current keyboard layout.

### Arguments

The program supports the following command-line arguments:

- `-d, --daemon`: Start Capsense in the background.
- `-s, --stop`: Stop running instance of Capsense.
- `-r, --reload`: Reload the configuration from `config.toml` for the running instance.
- `-S, --status`: Check if a Capsense instance is running and show its PID.
- `--startup <enable|disable>`: Enable or disable the program starting automatically with Windows.

## Configuration

On first run, a `config.toml` file will be created in the same directory. You can customize the following:

- `tap_threshold_ms`: The maximum duration (in milliseconds) for a `CapsLock` press to be considered a "tap". `300` ms
  by default.
- `tap_action`: The action to perform on a tap. Supported actions are:
  - `shortcut`: Trigger a keyboard shortcut (defined by `tap_shortcut`).
  - `switch_layout`: (Default) Rotate through input layouts.
- `tap_shortcut`: The shortcut to trigger (`["LWIN", "SPACE"]` by default). Supported keys are:
    - `LWIN` (or `WIN`)
    - `SPACE`
    - `LCONTROL` (or `CTRL`)
    - `LSHIFT` (or `SHIFT`)
    - `LMENU` (or `ALT`)
    - `CAPSLOCK`
- `layouts`: A list of input layout IDs to rotate through when `tap_action` is set to `switch_layout`.
  - Default: `[0x0804, 0x0409]` (`zh-CN` and `en-GB`).
  - See [Microsoft's documentation](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-lcid/70feba9f-294e-491e-b6eb-56532684c37f) for more layout IDs. Other common ones are:
    - `0x0404`: Traditional Chinese
    - `0x0411`: Japanese
    - `0x0412`: Korean
- `no_en`: When enabled, Capsense prevent your Chinese IMEs from entering English mode after layout or focus changes.
  `true` by default.

## License

```
Capsense is free software: you can redistribute it and/or modify 
it under the terms of the GNU General Public License as published 
by the Free Software Foundation, either version 3 of the License, 
or (at your option) any later version.

Capsense is distributed in the hope that it will be useful, 
but WITHOUT ANY WARRANTY; without even the implied warranty of 
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the 
GNU General Public License for more details.

You should have received a copy of the GNU General Public License 
along with this program. If not, see <https://www.gnu.org/licenses/>.
```
