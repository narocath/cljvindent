# cljvindent-emacs

`cljvindent-emacs` is an Emacs package built on top of `cljvindent`, an indentation and alignment engine for Clojure, ClojureScript, and EDN.

`cljvindent` is written in Rust and was created for speed, especially when indenting large regions and whole file buffers.

`cljvindent-emacs` builds the core engine as a native Emacs module and uses it to indent source code directly from Emacs. By default, the package offers to build the native module automatically on first use.

Some form indentation follows a specific style and includes a few mild layout preferences, but nothing too extreme.

## Features

- Indent the current form at point
- Indent the parent form at point
- Indent the outer parent form at point
- Indent the top-level form at point
- Indent the active region
- Indent the whole file

### Supported forms

- `let` and related binding forms, such as `loop` and `with-redefs`
- `cond`
- `condp`
- `as->`
- threading forms such as `->`, `->>`, `some->`, `some->>`, `cond->`, and `cond->>`
- `ns` forms, including ordering entries in `:require`, `:import`, and `:use` from shorter to longer
- maps
- vectors
- other common Clojure forms

#### Notes

- comments `;;` are ignored
- unevaluated forms such as `#_` are ignored

## Requirements

- Emacs 29.1+
- Rust
- Cargo available in `PATH`

## Installation

Install `cljvindent-emacs` from MELPA.

On first use, the package will offer to build the native module automatically.

## Usage

### Emacs

Available commands:

- `M-x cljvindent-current-form`
- `M-x cljvindent-parent`
- `M-x cljvindent-outer-parent`
- `M-x cljvindent-top-level-form`
- `M-x cljvindent-region`
- `M-x cljvindent-whole-buffer`

#### Customization

Useful options:

| Option | Default | Notes |
|---|---:|---|
| `cljvindent-build-command` | — | Full command used to build the native module |
| `cljvindent-auto-build-module` | `t` | Build the module automatically on first use |
| `cljvindent-enable-logs` | `nil` | Enable logging |
| `cljvindent-log-level` | `info` | Log level |
| `cljvindent-log-file-output-type` | `compact` | Choices: `compact`, `json` |
| `cljvindent-clean-after-build` | `t` | Clean build artifacts after build |

##### Notes
When enabled, logs are written relative to Emacs' current working directory under the folder `.cljvindent_logs/`, which often means they end up under the user's home directory.

#### Manual module installation

You can also build and load the module manually:

- `M-x cljvindent-install-module`

To force a rebuild:

- `M-x cljvindent-rebuild-module`

## Notes

The Rust native module is built locally and then loaded by Emacs from the installed package directory.

## License
Copyright © 2026 Panagiotis Koromilias

Licensed under the Apache License, Version 2.0.
