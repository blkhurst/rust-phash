# Rust

### Cargo Commands

* `cargo new <name>` - Create a new package from scratch.
* `cargo clean` - Remove the `target/` directory to force a fresh build.
* `cargo check` - Fast compile to check for errors.
* `cargo run` - Build (if needed) and run the project.
* `cargo build --release` - Optimized build for final binary.
* `cargo update` - Update dependencies in `Cargo.lock` to the latest allowed versions.
* `cargo test` - Run tests.
* `cargo fmt` - Auto-format code (`rustfmt` must be installed).
* `cargo clippy` - Extra lint checks (`clippy` must be installed).

## Rust Printing Variables

- `{}` - Uses `Display`; user-friendly formatting.
- `{:?}` - Uses `Debug`; compact developer-facing output.
- `{:#?}` - Uses `Debug`; pretty-printed, multi-line developer output.
- `{:x}` / `{:X}` - Uses `Display`; hexadecimal (lowercase/uppercase).
- `{:b}` - Uses `Display`; binary representation.
- `{:>n}` / `{:^n}` / `{:<n}` - Uses `Display`; right/center/left alignment with width `n`.
- `{:0n}` - Uses `Display`; zero-padded to width `n`.

## Rust Attributes

- `#[...]` - Attribute syntax; attaches compile-time metadata or configuration to the next item.
- `#[derive(...)]` - Auto-implement listed traits (e.g., `Debug`, `Clone`, `Serialize`) for the type.
- `#[cfg(...)]` - Conditional compilation; include code only if a feature/condition is met.
- `#[test]` - Marks a function as a test, run by `cargo test`.
- `///` - Sets documentation text.

## Rust Loops & Macro Syntax

- `loop { ... }` - Infinite loop; break with `break` when needed.
- `while condition { ... }` - Loop while condition is true.
- `for x in iterable { ... }` - Loop over items in an iterator.
- `!` after a name - Calls a macro, not a function (e.g., `println!`, `format!`, `vec!`).
