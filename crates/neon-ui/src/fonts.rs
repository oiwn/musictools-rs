//! Bundled typography: Inter Medium for body text and labels, Inter SemiBold
//! for headings. Inter is licensed under the SIL Open Font License 1.1
//! (`assets/fonts/OFL.txt`), which allows embedding in commercial software as
//! long as the license ships with it and the font is not sold on its own.

use egui::epaint::text::{FontData, FontDefinitions};
use egui::{Context, FontFamily, FontId};
use std::sync::Arc;

const INTER_MEDIUM: &[u8] = include_bytes!("../assets/fonts/Inter-Medium.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../assets/fonts/Inter-SemiBold.ttf");

const BODY: &str = "Inter-Medium";
const HEADING: &str = "Inter-SemiBold";
/// Family name for headings; use [`heading`] rather than spelling it out.
pub const HEADING_FAMILY: &str = "neon-heading";

/// The kit's font definitions: Inter first in `Proportional`, a heading
/// family, and egui's own fonts kept as fallback for missing glyphs.
#[must_use]
pub fn definitions() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        BODY.to_owned(),
        Arc::new(FontData::from_static(INTER_MEDIUM)),
    );
    fonts.font_data.insert(
        HEADING.to_owned(),
        Arc::new(FontData::from_static(INTER_SEMIBOLD)),
    );

    let fallback = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, BODY.to_owned());
    let mut heading_chain = vec![HEADING.to_owned(), BODY.to_owned()];
    heading_chain.extend(fallback);
    fonts
        .families
        .insert(FontFamily::Name(HEADING_FAMILY.into()), heading_chain);
    fonts
}

/// Install the kit's fonts. Call once when the editor or app starts; egui
/// applies them from the next frame.
pub fn install(ctx: &Context) {
    ctx.set_fonts(definitions());
}

/// A heading font of `size` px (Inter SemiBold).
#[must_use]
pub fn heading(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(HEADING_FAMILY.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Run two frames so `set_fonts` takes effect, then query glyph coverage.
    fn with_installed_fonts(check: impl FnOnce(&Context)) {
        let ctx = Context::default();
        install(&ctx);
        for _ in 0..2 {
            let mut output = ctx.run_ui(egui::RawInput::default(), |_| {});
            output.textures_delta.clear();
        }
        check(&ctx);
    }

    #[test]
    fn inter_leads_the_proportional_family() {
        let fonts = definitions();
        assert_eq!(fonts.families[&FontFamily::Proportional][0], BODY);
        assert_eq!(
            fonts.families[&FontFamily::Name(HEADING_FAMILY.into())][0],
            HEADING
        );
    }

    #[test]
    fn kit_glyphs_resolve_in_body_and_heading_fonts() {
        with_installed_fonts(|ctx| {
            ctx.fonts_mut(|fonts| {
                for font in [FontId::proportional(11.0), heading(14.0)] {
                    assert!(
                        fonts.has_glyphs(&font, "AZaz09%-.·×—●"),
                        "missing kit glyphs in {font:?}"
                    );
                }
            });
        });
    }
}
