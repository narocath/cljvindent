# cljvindent

`cljvindent` is a Rust indentation and vertical-alignment engine for Clojure, ClojureScript, and EDN code. It's created for speed and for handling large regions, large files. Its core library can be used either through the CLI as a standalone executable that can accept either a string or a path for a whole file or through an Emacs package that loads it as a native module. Some form alignments follow a specific style and include a few mild layout preferences, but nothing too extreme.

## Features

- Format the current form at point
- Format the parent form at point
- Format the outer parent form at point
- Format the top-level form at point
- Format the active region
- Format the whole file

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
- comments are ignored
- unevaluated forms such as `#_` are ignored

## Requirements

- Emacs 29.1+
- Rust
- Cargo available in `PATH`

## Installation

## Usage

### Emacs

Available commands:

- `M-x cljvindent-format-current-form`
- `M-x cljvindent-format-parent`
- `M-x cljvindent-format-outer-parent`
- `M-x cljvindent-format-top-level-form`
- `M-x cljvindent-format-region`
- `M-x cljvindent-align-whole-buffer`

#### Customization

Useful options:

- `cljvindent-cargo-command`
- `cljvindent-auto-build-module`
- `cljvindent-enable-logs`
- `cljvindent-log-level`
- `cljvindent-log-file-output-type`

#### Manual module installation

You can also build and load the module manually:

- `M-x cljvindent-install-module`

To force a rebuild:

- `M-x cljvindent-rebuild-module`

## Notes

The Rust native module is build locally and then loaded by Emacs from the installed package directory.


## TODO
- add more documentation
- add more tests
- integrate with Git to support formatting hunks through different hooks
- publish the CLI crate on crates.io
- add a VS Code extension
- add support for `doseq`, `for`



## License
Copyright © 2026 Panagiotis Koromilias

Licensed under the Apache License, Version 2.0.
