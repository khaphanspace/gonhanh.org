# Gõ Nhanh - Linux (Fcitx5)

Vietnamese Input Method Engine for Linux using Fcitx5 framework.

## Requirements

### Build dependencies
```bash
# Ubuntu/Debian
sudo apt install cmake pkg-config fcitx5 fcitx5-dev libfcitx5core-dev \
    libfcitx5config-dev libfcitx5utils-dev libxkbcommon-dev

# Fedora
sudo dnf install cmake pkg-config fcitx5 fcitx5-devel libxkbcommon-devel

# Arch Linux
sudo pacman -S cmake pkg-config fcitx5 xkbcommon
```

### Runtime dependencies
- Fcitx5 (input method framework)
- Rust core library (bundled)

## Building

```bash
# From project root
cd platforms/linux
./scripts/build.sh

# Debug build
./scripts/build.sh --debug
```

## Installation

### User-local (recommended, no sudo)
```bash
./scripts/install.sh
```

### System-wide
```bash
cd build
sudo make install
```

## Configuration

1. Add library path to your shell config (`~/.bashrc` or `~/.zshrc`):
   ```bash
   export LD_LIBRARY_PATH="$HOME/.local/lib:$LD_LIBRARY_PATH"
   ```

2. Restart Fcitx5:
   ```bash
   fcitx5 -r &
   ```

3. Open Fcitx5 configuration:
   ```bash
   fcitx5-configtool
   ```

4. Add "Gõ Nhanh" to your input methods

## Usage

- **Telex mode** (default):
  - Diacritics: `s`=sắc, `f`=huyền, `r`=hỏi, `x`=ngã, `j`=nặng
  - Vowels: `aa`=â, `oo`=ô, `ee`=ê, `aw`=ă, `ow`=ơ, `uw`=ư, `w`=ư
  - Example: `vieetj` → việt

- **VNI mode**:
  - Diacritics: `1`=sắc, `2`=huyền, `3`=hỏi, `4`=ngã, `5`=nặng
  - Vowels: `6`=â/ô/ê, `7`=ư/ơ, `8`=ă
  - Example: `vie65t` → việt

## Shortcuts

Use Fcitx5's built-in shortcuts to switch input methods (default: Ctrl+Space).

## File Locations

| File | Location |
|------|----------|
| Addon library | `~/.local/lib/fcitx5/gonhanh.so` |
| Addon config | `~/.local/share/fcitx5/addon/gonhanh.conf` |
| IM config | `~/.local/share/fcitx5/inputmethod/gonhanh.conf` |
| Rust core | `~/.local/lib/libgonhanh_core.so` |

## Troubleshooting

### Addon not loading
```bash
# Check Fcitx5 logs
fcitx5 --verbose=*:4

# Verify library can be loaded
ldd ~/.local/lib/fcitx5/gonhanh.so
```

### Missing symbols
```bash
# Check LD_LIBRARY_PATH
echo $LD_LIBRARY_PATH

# Should include ~/.local/lib
```

## License

GPL-3.0-or-later (same as Fcitx5)
