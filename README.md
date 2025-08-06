# Steam Icon Recovery
A simple rust application to recover lost icons for steam `.desktop` file
shortcuts on linux. All it does is parse the game id, find the corresponding
The application functions as follows:
1. Parse game ID from shortcut file
2. Find corresponding image file using [steamcmd](https://developer.valvesoftware.com/wiki/SteamCMD)
3. Download the icon file and split it into various pngs
4. Set the icon in the shortcut file

## Usage
```bash
steam-icon-recovery [OPTIONS]
```

### Options:
| Flag                  | Description                                                                              |
| --------------------- | ---------------------------------------------------------------------------------------- |
| `-f`, `--file <FILE>` | Process a specific `.desktop` file (overrides `--dir`)                                   |
| `-d`, `--dir <DIR>`   | Directory to search for `.desktop` files (defaults to `$HOME/.local/share/applications`) |
| `-h`, `--help`        | Print help information                                                                   |
| `-V`, `--version`     | Show version information                                                                 |

### Examples
- Process specific file:
```bash
steam-icon-recovery --file ~/Games/game-shortcut.desktop
```
- Process all `.desktop` files in a given directory:
```bash
steam-icon-recovery --file ~/Games/applications
```
- Use defaults, process all `.desktop` files in `~/.local/share/applications`:
```bash
steam-icon-recovery
```
