# Ideas

Uncommitted possibilities — noted but not decided to do right now. Promote to `roadmap.md` only when we decide to pursue something.

## Grey out Bogdan controls the current Mode ignores

Shape (Sine / Triangle) only does anything in Fold mode, but the dropdown looks live in Clean and Detail — it invites a change that produces no sound. Same for the other two dependent controls:

- **Clean** — Drive and Ceiling live; Detail, Amount, and Shape inert.
- **Detail** — Drive, Ceiling, Detail, and Amount live; Shape inert.
- **Fold** — Drive, Ceiling, Amount, and Shape live; Detail inert.

(In Fold the delta high-pass still runs every sample so its memory stays warm, but its output never reaches the output — so the Detail knob is inert there.)

nice-plug has no per-parameter "inactive" flag either (`ParamFlags` covers automation, hiding, and bypass), so this is editor-side only: put the affected controls in a child `Ui` and call `ui.disable()`, as Bogdan's editor already does for SHAPE outside Fold (0006). Still open: Detail and Amount in Clean, Detail in Fold.

Open questions: does the parameter stay automatable while greyed (it should — hiding it from the host would break existing automation), and is greying enough or should the row reflow? Kirya has the same situation with Mod Rate / Depth / Shape at Mod Depth 0, so whatever pattern lands here should suit both.

## Move the offline Analyzer into musictools-core

Kirya's `analysis::Analyzer` (envelope, spectrogram, RT60) and igorek's `ir_probe` spectrogram now both hand-roll the same ASCII FFT analysis with `realfft` — two concrete consumers, which is the house rule's threshold for promotion. A shared `musictools-core` analyzer (behind a feature so the core crate stays dependency-free for the audio path) would let both probes and any future plugin share one implementation.
