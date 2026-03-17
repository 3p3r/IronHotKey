# IronHotKey

IronHotKey is a port of AutoHotKey v1 to cross-platform Rust.

## Features

- Tree-Sitter for parsing AutoHotKey v1 scripts.
- Cross-platform support (Windows, macOS, Linux).
- TypeScript / JavaScript code generation for scripts.

## Architecture

This project at high level transpiles AutoHotKey v1 scripts to JavaScript, and executes them on the fly.

The utilities offered by IronHotKey can be used in any other JavaScript environment with any runtime, such as Node.js or Deno.

This project only offers the bare minimum runtime to execute the generated JavaScript code, and does not include any additional utilities or libraries.

The following modules are provided as part of the runtime, available to other runtimes via `napi-rs`, and directly through the runtime offered by IronHotKey:

- Environment (`ironhotkey/env`)
- External Libraries (`ironhotkey/ext`)
- File, Directory and Disk (`ironhotkey/disk`)
- Flow of Control (`ironhotkey/flow`)
- Graphical User Interfaces (`ironhotkey/gui`)
- Maths (`ironhotkey/maths`)
- Mouse and Keyboard (`ironhotkey/mnk`)
- Misc. (`ironhotkey/misc`)
- Object Types (`ironhotkey/types`)
- Process (`ironhotkey/process`)
- Registry (`ironhotkey/registry`)
- Screen (`ironhotkey/screen`)
- Sound (`ironhotkey/sound`)
- String (`ironhotkey/string`)
- Window (`ironhotkey/window`)
- #Directives (`ironhotkey/directives`)

## Language Reference

The language reference from the original AutoHotKey v1 `.chm` file, unpacked to `.html` is checked in under `reference`, and can be found at [reference/docs/index.htm](reference/docs/index.htm) and its index can be found at [reference/Index.hhk](reference/Index.hhk).

## References

- [AutoHotKey Parser](https://github.com/alfredomtx/tree-sitter-autohotkey)
- [AutoHotKey v1 API](https://www.autohotkey.com/docs/v1/)
- [JavaScript engine](https://github.com/DelSkayn/rquickjs)
- [NAPI-RS framework](https://napi.rs)
