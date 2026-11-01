# Gõ Nhanh - Linux (Fcitx5)

Fcitx5 addon backed by the shared Rust engine.

## Runtime support

- Stable release asset: Linux x86_64, `gonhanh-linux.tar.gz`
- Source builds: any Linux architecture supported by Rust, Fcitx5 and this C++ addon
- Fcitx5 is required at runtime

## Build dependencies

```bash
# Ubuntu/Debian
sudo apt install cmake pkg-config fcitx5 libfcitx5core-dev \
  libfcitx5config-dev libfcitx5utils-dev fcitx5-modules-dev \
  libxkbcommon-dev libgtest-dev

# Fedora
sudo dnf install cmake pkg-config fcitx5 fcitx5-devel \
  libxkbcommon-devel gtest-devel

# Arch Linux
sudo pacman -S cmake pkg-config fcitx5 xkbcommon gtest
```

Rust stable is also required.

## Build and test

```bash
./platforms/linux/scripts/build.sh
./platforms/linux/scripts/test.sh
```

Equivalent explicit commands:

```bash
cargo build --manifest-path core/Cargo.toml --release
cmake -S platforms/linux -B platforms/linux/build \
  -DCMAKE_BUILD_TYPE=Release -DBUILD_TESTS=ON
cmake --build platforms/linux/build --parallel
ctest --test-dir platforms/linux/build --output-on-failure
bash platforms/linux/tests/InstallerTest.sh
```

The bridge contract mirrors Rust `Result`: 256 UTF-32 codepoints followed by
`action`, `backspace`, `count`, and `flags`. Layout assertions and real FFI
composition/maximum-output tests guard this boundary.

## User-local installation

The canonical installer is safe to repeat and preserves existing Fcitx entries:

```bash
bash platforms/linux/scripts/install.sh
source ~/.profile
gn on
```

`make install-user` invokes the same script.

Installed files:

| File | Location |
|------|----------|
| Addon library | `~/.local/lib/fcitx5/gonhanh.so` |
| Rust core | `~/.local/lib/libgonhanh_core.so` |
| Addon config | `~/.local/share/fcitx5/addon/gonhanh.conf` |
| Input method config | `~/.local/share/fcitx5/inputmethod/gonhanh.conf` |
| CLI | `~/.local/bin/gn` |
| Installer/version | `~/.local/share/gonhanh/` |

The installer appends one GoNhanh item to the first Fcitx group. It does not
replace `DefaultIM` or remove Bamboo/other engines. It also writes one managed
Fcitx/PATH block to `~/.profile`.

## Package

After a release build:

```bash
bash platforms/linux/scripts/package.sh 1.2.3 ./gonhanh-linux.tar.gz
tar -tzf ./gonhanh-linux.tar.gz
```

The archive always contains a `gonhanh-linux/` root and its executable
`install.sh`. Both stable and pre-release workflows use this script.

## Troubleshooting

```bash
fcitx5 -r
fcitx5-configtool
ldd ~/.local/lib/fcitx5/gonhanh.so
fcitx5 --verbose=*:4
```

## License

GPL-3.0-or-later (same as Fcitx5)
