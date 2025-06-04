# Swaykbdd-rs: Automatic keyboard layout switcher in [Sway](https://swaywm.org/)

The swaykbdd-rs is fork of [swaykbdd](https://github.com/artemsen/swaykbdd) rewritten
in rust.

The utility can be used to automatically change the keyboard
layout on a per-window basis.

## Installation and usage

`cargo install --git https://github.com/dydyamotya/swaykbdd-rs.git`

Just run `$HOME/.cargo/bin/swaykbdd-rs`.
For automatic startup add this command to Sway config file:
`exec $HOME/.cargo/bin/swaykbdd-rs`
