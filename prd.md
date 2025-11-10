# PRD - Brut Terminal Editor

## TL;DR
- Product: brut, a single-file Rust terminal text editor focused on predictable keystroke handling.
- Target users: Developers or operators who work inside remote shells and need a reliable fallback editor.
- Objective: Ship a distraction-free editing core (open, insert, delete, move, save, quit) with minimal UI chrome.

## Background
Remote servers and disposable development environments rarely ship with the same editors. Some minimal tools (for example nano or ed) have inconsistent keymaps or are missing entirely. brut fills this gap with a tiny binary that relies only on a POSIX-like terminal (via crossterm) and handles large buffers through ropey.

## Goals
1. Provide deterministic keyboard-driven editing that mirrors familiar cursor semantics.
2. Support both editing existing files and capturing scratch notes without specifying a filename up front.
3. Keep the codebase compact and dependency-light to simplify auditing and distribution.
4. Offer a clear quit-and-save flow that prevents accidental data loss.

## Non-goals
- Syntax highlighting, search and replace, multiple buffers, or plugins.
- Mouse input or window splitting.
- Network awareness or collaborative editing.

## Personas and User Stories
- Incident responder: "When I ssh into a host, I can open config files with brut file.cfg, make edits, and save quickly without learning a new UI."
- Scratchpad user: "I can run brut, jot notes, and when I hit Esc I am prompted for a filename so I can persist the buffer if desired."
- Minimalist developer: "Arrow keys should behave the same way as in mainstream editors with no jumping surprises."

## Functional Requirements
1. **Editing loop**
   - Insert printable characters at the cursor while keeping the rope-backed buffer consistent.
   - Support newline insertion via Enter that advances the logical line index before redrawing.
   - Handle Backspace by removing the previous character when available.
2. **Navigation**
   - Left and Right move within the current line while respecting buffer bounds and updating the preferred column.
   - Up and Down move vertically, clamping to the shorter of the target line length or the preferred column stored on the editor.
3. **File I/O**
   - When a filename is passed on launch, preload its contents; otherwise start with an empty buffer and mark the session as a new file.
   - Saving writes the rope contents to disk using UTF-8 and, for new files, adopts the provided filename for future saves.
4. **Quit and save prompt**
   - Pressing Esc triggers a prompt line showing "Filename: (buffer contents)" plus instructions to press Enter to save or Esc to discard.
   - Accept filename characters, handle Backspace, and position the cursor after the prompt text for clarity.
5. **Terminal UX**
   - Use EnterAlternateScreen and LeaveAlternateScreen, clear the screen on each draw, and reposition the cursor via MoveTo(x, y) so the terminal mirrors the in-memory cursor.
6. **Responsiveness**
   - Poll input with a 200 ms tick to keep the UI responsive without busy-waiting and leave room for future background tasks (status line, autosave, etc.).

## Non-functional Requirements
- Portability: Runs wherever crossterm supports (Windows, macOS, Linux) without platform-specific forks.
- Robustness: Any I/O failure should fail loudly rather than silently discarding data.
- Code simplicity: Maintain a single source file (src/main.rs) so onboarding stays trivial.

## Success Metrics
- Cold start under 50 ms on commodity hardware (release build).
- Editing operations remain responsive (input to screen update under 50 ms) for at least 1 MB files thanks to rope-backed storage.
- Zero regressions in save and quit flow during manual smoke tests.

## Release Checklist
- [ ] Manual test matrix (Linux, macOS, Windows) covering launch without args, editing an existing file, save prompt cancel, and save prompt confirm.
- [ ] Provide a usage snippet in the README showing keybindings.
- [ ] Tag v0.1.0 once manual checks are complete and binary size is recorded.

## Open Questions and Future Enhancements
- Optional read-only mode to preview files safely.
- Configurable key bindings for Emacs or Vim users.
- Status bar with current line and column plus dirty indicator.
- Integration tests exercising key paths via a pseudo-terminal harness.
