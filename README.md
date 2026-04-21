# ink-rs

`ink-rs` aims to port the official C# ink compiler layer to Rust.

The runtime layer is reused from the local `blade-ink-rs/lib` crate. The local
`ink-csharp/` tree is the architecture and naming reference for the compiler
port, especially `ink-csharp/compiler`.

Both reference trees are intentionally ignored by this repository. Keep them
next to this project when building locally.

## Layout

- `crates/ink-compiler`: Rust compiler layer under development.
- `ink-csharp/`: local official C# reference implementation, ignored by Git.
- `blade-ink-rs/`: local Rust runtime implementation, ignored by Git.

## Current Status

The project currently contains the initial workspace and compiler crate
scaffold. Parser, parsed hierarchy, reference resolution, and runtime export are
to be ported from the C# compiler.

