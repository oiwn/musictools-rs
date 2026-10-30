---
id: 0018-igorek-nice-port-docs
status: done
scope: [plugins/igorek/, README.md, AGENTS.md, .gitignore, crates/neon-ui/src/visuals.rs, crates/musictools-core/src/rt_audit.rs]
created: 2026-10-29
source: user request 2026-10-29: igorek port plus tooling/docs sweep
depends: [0017-kirya-nice-port]
---
# Task: igorek nice port docs

## Plan

- [x] Delete `plugins/igorek/` (the last truce code) and the now-unused `rfd` workspace dependency; `crates/igorek-dsp` stays.
- [x] `specs/truce_overview.md` → `specs/nice_overview.md`: a nice-plug 0.4 cheat-sheet written from the Bogdan and Kirya ports, linked from overview.md.
- [x] Specs: overview.md, igorek.md (status note: wrapper removed, DSP kept), roadmap.md, ideas.md re-pointed at nice-plug.
- [x] Drafts 0009, 0013, 0014 re-pointed at nice-plug (ClapPlugin metadata, `cargo nice-plug bundle`, `rt-audit`, `#[persist]`).
- [x] README.md and AGENTS.md: nice-plug build/install commands, Igorek wrapper removed.
- [x] Code comments: `neon-ui/src/visuals.rs`, `musictools-core/src/rt_audit.rs`, `.gitignore`; legacy-parity notes in Kirya/Bogdan stay.

## Acceptance

- [x] `cargo fmt --all --check`
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- [x] `cargo test --workspace --all-features`
- [x] `typos`
- [x] `gitleaks git --verbose`
- [x] `cargo nice-plug bundle ppc_bogdan --release`
- [x] `cargo nice-plug bundle ppc_kirya --release`

## Manual checks

- [x] README build and install steps work as written. — waived by user: "Looks good let's close" (bundle commands run by agent; `just deploy` not run)

## Context

- Final migration task; completes the workspace on nice-plug/egui 0.37 and cleans every remaining truce reference.
- Re-planned 2026-10-30. User: "Finish Truce removal", and for Igorek "Delete the wrapper" instead of porting it. Igorek was already out of the release set (decision 2026-10-29); its DSP crate builds truce-free and stays, so a revival writes a fresh nice-plug wrapper (neon, Kirya-style, per the user).
- Truce mentions that stay on purpose: Kirya/Bogdan comments and tests pinning parity with the legacy truce builds, which stay installed beside the `ppc_*` ones.

## Review

- `plugins/igorek/` deleted (git history keeps it); `rfd` dropped from workspace deps (nothing else used it; it was never in `Cargo.lock`). `igorek-dsp` still builds and tests.
- `specs/nice_overview.md` replaces `truce_overview.md`, built from the Bogdan and Kirya code. `#[persist]` is described but unused in the repo, and flagged "verify on first use".
- Final sweep `rg -il truce -g '!target' -g '!CHANGELOG.md' -g '!specs/tasks/done/**'` finds history only:
  - migration notes in overview, roadmap and AGENTS;
  - legacy-build parity in the Kirya and Bogdan wrappers and `kirya.md`;
  - the `audio-harness` module doc ("successor to truce's …", out of scope);
  - truce-era notes in `igorek.md` under its new status note;
  - old 0014 log lines;
  - this task.
- Out of scope: the 29 earlier uncommitted files sit in the baseline, untouched.
- The README's bundle commands ran here; `just deploy` installs into `~/Library` and is left to the user.

## Summary

Truce removed: the Igorek wrapper deleted (DSP kept), a nice-plug cheat-sheet replaces the truce one, and README, specs and drafts re-pointed at nice-plug.

## Log

- 2026-10-29 created: igorek nice port docs
- 2026-10-29 set source user request 2026-10-29: igorek port plus tooling/docs sweep
- 2026-10-29 scope add plugins/igorek/ — igorek port and framework docs/tooling sweep
- 2026-10-29 scope add README.md — igorek port and framework docs/tooling sweep
- 2026-10-29 scope add AGENTS.md — igorek port and framework docs/tooling sweep
- 2026-10-29 scope add specdev.toml — igorek port and framework docs/tooling sweep
- 2026-10-29 set depends 0017-kirya-nice-port
- 2026-10-30 scope rm specdev.toml — no truce entries left in specdev.toml
- 2026-10-30 scope add .gitignore — stale truce wording in comments
- 2026-10-30 scope add crates/neon-ui/src/visuals.rs — stale truce wording in comments
- 2026-10-30 scope add crates/musictools-core/src/rt_audit.rs — stale truce wording in comments
- 2026-10-30 draft approved: Finish Truce removal; Igorek: Delete the wrapper (user, 2026-10-30)
- 2026-10-30 advance draft → ready
- 2026-10-30 scope approved: plugins/igorek/, README.md, AGENTS.md, .gitignore, crates/neon-ui/src/visuals.rs, crates/musictools-core/src/rt_audit.rs
- 2026-10-30 advance ready → in-progress/implement
- 2026-10-30 baseline: crates/audio-harness/src/lib.rs@36e82b5987ab, crates/musictools-core/src/lib.rs@f3092ee5532a, crates/musictools-core/src/units.rs@da3a8a279f65, crates/neon-ui/assets/fonts/Inter-Medium.ttf@458cd0601d28, crates/neon-ui/assets/fonts/Inter-SemiBold.ttf@47f8ab1d6814, crates/neon-ui/assets/fonts/OFL.txt@9b2ca37b3ffc, crates/neon-ui/examples/common/sections.rs@90b35b47802d, crates/neon-ui/examples/showcase.rs@cd98936639b1, crates/neon-ui/examples/snapshot.rs@c3199d0df5c5, crates/neon-ui/src/bindings.rs@d3bfe2758262, crates/neon-ui/src/button.rs@dfeb559d8e27, crates/neon-ui/src/fonts.rs@fe9c7118b8a7, crates/neon-ui/src/glow.rs@9ed09cc2c24e, crates/neon-ui/src/knob.rs@cee5c93e9d12, crates/neon-ui/src/lib.rs@7591d17c893d, crates/neon-ui/src/slider.rs@8e426330d6f7, crates/neon-ui/src/surface.rs@44fc99ed22e9, crates/neon-ui/src/tabs.rs@1b0ab9394a39, crates/neon-ui/src/text.rs@337c8083e44d, justfile@98d7535da77b, plugins/bogdan/Cargo.toml@548fa77e8f7c, plugins/bogdan/src/editor.rs@8314aa5c2301, plugins/bogdan/src/lib.rs@556a1a63932d, plugins/kirya/Cargo.toml@af3105b93fe3, plugins/kirya/src/editor.rs@b051c1ff59c4, plugins/kirya/src/lib.rs@c40db5397115, plugins/kirya/src/task.rs@47cf84f1339c, prek.toml@1aaf4748f62f, specdev.toml@85e705ffe263
- 2026-10-30 advance in-progress/implement → in-progress/verify
- 2026-10-30 advance in-progress/verify → in-progress/review
- 2026-10-30 advance in-progress/review → approval
- 2026-10-30 approved: Looks good let's close (user, 2026-10-30)
- 2026-10-30 advance approval → done
