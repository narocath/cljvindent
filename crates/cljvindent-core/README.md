# cljvindent-core

`cljvindent-core` is the core library behind `cljvindent`, used by both the CLI and the Emacs package. It is focused on fast indentation and alignment for Clojure(script), and EDN, especially for large regions and whole files.

Main repository [here](https://github.com/narocath/cljvindent).

## Supported forms

- `let` and related binding forms such as `loop` and `with-redefs`
- `cond`
- `condp`
- `as->`
- threading forms such as `->`, `->>`, `some->`, `some->>`, `cond->`, and `cond->>`
- `ns` forms
- maps
- vectors
- other common Clojure forms

## Notes

- comments are ignored
- unevaluated forms such as `#_` are ignored


## License
Copyright © 2026 Panagiotis Koromilias

Licensed under the Apache License, Version 2.0.
