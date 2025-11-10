# brut

A single-file Rust terminal editor that behaves predictably on any machine with a modern terminal. It aims to be a muscle-memory-friendly fallback that lands somewhere between VS Code's straightforwardness and Vim's snappy keystroke loop.

## Features
- Rope-backed buffer via `ropey` keeps inserts/deletes responsive even on large files.
- Crossterm-driven UI works the same on Linux, macOS, and Windows (including WSL/PowerShell).
- Arrow-key navigation with sticky columns, newline insertion, and backspace that stitches lines like mainstream editors.
- Escape-driven save prompt so you can launch as a scratchpad and decide later whether to persist.

## Usage
```bash
# run against an existing file
cargo run -- path/to/file.txt

# or start a scratch buffer (you'll be prompted for a name on save)
cargo run --
```

### Keybindings
- `text` keys: insert at the cursor.
- `Enter`: insert a newline and move to the next line.
- `Backspace`: delete left; at column 0 it merges with the previous line.
- `←/→`: move horizontally, wrapping to previous/next line edges.
- `↑/↓`: move vertically while preserving the preferred column when possible.
- `Esc`: open the save/discard prompt. In the prompt, type a filename (only required for brand-new buffers), press `Enter` to save, or `Esc` again to quit without saving.

## Project Docs
- `prd.md` – product requirements and release checklist.
- `agents.md` – who owns what (Product, Engineering, QA, Docs/Release) and how to hand work off.

## Next Steps
Planned improvements live in `prd.md`, but the shortlist includes a status bar, configurable keymaps, and a richer manual test matrix. Contributions welcome—open an issue describing the behavior you want to add or tighten up.
