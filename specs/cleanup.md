# Cleanup

Code smells, duplication, and refactor notes. Read on demand only — not session context. Each entry is a `##` heading with the file or component, followed by 1-3 sentences.

## neon-ui select popup alignment

`neon-ui/src/select.rs` opens its menu with egui's default `BOTTOM_START` alignment plus auto-flip near window edges, which jumps horizontally in real plugin hosts (seen in ppc_bogdan under Serum 2, 2026-10-30). Fix with `Popup::align`/`align_alternatives` or an inline expanding menu when 0006/0007 adopt NeonSelect; plugins currently prefer `param_tabs` (no popups).
