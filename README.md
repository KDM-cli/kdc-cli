# KDC

Kubernetes Docker Commander is a project-centric DevOps terminal application.

The current implementation is the initial foundation slice from the design docs:

- project scanning
- capability generation
- dynamic menu generation
- startup state
- a keyboard-first Ratatui dashboard shell

Run locally once Rust is installed:

```bash
cargo run
cargo run -- scan
cargo run -- menus
cargo test
```

