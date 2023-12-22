# Dotman
Dotfile manager
## Description
Dotman is your simple dotfiles manager.
## Installation
```bash
git clone https://github.com/ryohz/dotman.git
cd dotman
cargo build --release
```
## Usage
- Initializing: 
```bash
dotman init
```
- Adding your config directory
```bash
dotman add i3 ~/.config/i3/
```
- Exporting all your configs
```bash
dotman export
```
- Importing all configs of dotfiles to your machine
```bash
dotman import
```
## Hooks
You can config your script which will be run 
- before importing
- after importing
- after exporting.
### Example
```dotman.toml```
```toml 
after_import_hook = "/home/user/dotfiles/after_import_hook"
before_import_hook = "/home/user/dotfiles/before_import_hook"
export_hook = "/home/user/dotfiles/export_hook"

[[pairs]]
name = "i3"
place = "/home/user/.config/i3"
hash = 364712289

[[pairs]]
name = "wallpaper"
place = "/home/user/.config/wallpaper"
hash = 3450895380

[[pairs]]
name = "polybar"
place = "/home/user/.config/polybar/"
hash = 1364170016
```
```before_import_hook```
```shell
cd /home/user/dotfiles/
git fetch
git reset --hard origin/main
git pull origin main --force
```
```after_import_hook```
```shell
i3-msg restart
feh --bg-scale ~/.config/wallpaper/1.jpg
```
```export_hook```
```shell
cd /home/user/dotfiles/
git add .
git commit -m "commit"
git push origin main
```
