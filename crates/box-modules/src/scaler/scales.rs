//! The scale catalog. Every scale is a list of degrees in cents within the
//! octave, starting at 0 and staying below 1200, so microtonal scales sit
//! beside the 12-tone ones.

/// A named scale.
#[derive(Clone, Copy, Debug)]
pub struct Scale {
    pub name: &'static str,
    /// Degrees in cents, ascending, `0 <= degree < 1200`.
    pub cents: &'static [f32],
}

/// `steps` equal divisions of the octave.
const fn equal<const N: usize>() -> [f32; N] {
    let mut out = [0.0; N];
    let mut index = 0;
    while index < N {
        out[index] = 1200.0 * index as f32 / N as f32;
        index += 1;
    }
    out
}

const EDO_12: [f32; 12] = equal::<12>();
const EDO_19: [f32; 19] = equal::<19>();
const EDO_24: [f32; 24] = equal::<24>();

/// Every scale, in menu order.
pub const SCALES: [Scale; 35] = [
    // Diatonic modes.
    scale("Major", &[0.0, 200.0, 400.0, 500.0, 700.0, 900.0, 1100.0]),
    scale("Minor", &[0.0, 200.0, 300.0, 500.0, 700.0, 800.0, 1000.0]),
    scale("Dorian", &[0.0, 200.0, 300.0, 500.0, 700.0, 900.0, 1000.0]),
    scale(
        "Phrygian",
        &[0.0, 100.0, 300.0, 500.0, 700.0, 800.0, 1000.0],
    ),
    scale("Lydian", &[0.0, 200.0, 400.0, 600.0, 700.0, 900.0, 1100.0]),
    scale(
        "Mixolydian",
        &[0.0, 200.0, 400.0, 500.0, 700.0, 900.0, 1000.0],
    ),
    scale("Locrian", &[0.0, 100.0, 300.0, 500.0, 600.0, 800.0, 1000.0]),
    // Common.
    scale(
        "Harmonic minor",
        &[0.0, 200.0, 300.0, 500.0, 700.0, 800.0, 1100.0],
    ),
    scale(
        "Melodic minor",
        &[0.0, 200.0, 300.0, 500.0, 700.0, 900.0, 1100.0],
    ),
    scale("Major pentatonic", &[0.0, 200.0, 400.0, 700.0, 900.0]),
    scale("Minor pentatonic", &[0.0, 300.0, 500.0, 700.0, 1000.0]),
    scale("Blues", &[0.0, 300.0, 500.0, 600.0, 700.0, 1000.0]),
    scale("Whole tone", &[0.0, 200.0, 400.0, 600.0, 800.0, 1000.0]),
    scale(
        "Diminished W-H",
        &[0.0, 200.0, 300.0, 500.0, 600.0, 800.0, 900.0, 1100.0],
    ),
    scale(
        "Diminished H-W",
        &[0.0, 100.0, 300.0, 400.0, 600.0, 700.0, 900.0, 1000.0],
    ),
    scale("Chromatic", &EDO_12),
    // Exotic, 12-tone.
    scale(
        "Hungarian minor",
        &[0.0, 200.0, 300.0, 600.0, 700.0, 800.0, 1100.0],
    ),
    scale(
        "Double harmonic",
        &[0.0, 100.0, 400.0, 500.0, 700.0, 800.0, 1100.0],
    ),
    scale(
        "Phrygian dominant",
        &[0.0, 100.0, 400.0, 500.0, 700.0, 800.0, 1000.0],
    ),
    scale("Persian", &[0.0, 100.0, 400.0, 500.0, 600.0, 800.0, 1100.0]),
    scale(
        "Neapolitan major",
        &[0.0, 100.0, 300.0, 500.0, 700.0, 900.0, 1100.0],
    ),
    scale(
        "Neapolitan minor",
        &[0.0, 100.0, 300.0, 500.0, 700.0, 800.0, 1100.0],
    ),
    scale(
        "Enigmatic",
        &[0.0, 100.0, 400.0, 600.0, 800.0, 1000.0, 1100.0],
    ),
    scale("Hirajoshi", &[0.0, 200.0, 300.0, 700.0, 800.0]),
    scale("In-sen", &[0.0, 100.0, 500.0, 700.0, 1000.0]),
    scale("Iwato", &[0.0, 100.0, 500.0, 600.0, 1000.0]),
    scale("Kumoi", &[0.0, 200.0, 300.0, 700.0, 900.0]),
    scale("Egyptian", &[0.0, 200.0, 500.0, 700.0, 1000.0]),
    scale("Pelog (12-TET)", &[0.0, 100.0, 300.0, 700.0, 800.0]),
    // Microtonal.
    scale(
        "Maqam Rast",
        &[0.0, 200.0, 350.0, 500.0, 700.0, 900.0, 1050.0],
    ),
    scale(
        "Maqam Bayati",
        &[0.0, 150.0, 300.0, 500.0, 700.0, 800.0, 1000.0],
    ),
    scale("Slendro", &[0.0, 240.0, 480.0, 720.0, 960.0]),
    scale("Pelog", &[0.0, 120.0, 270.0, 540.0, 670.0, 780.0, 950.0]),
    scale("19-TET", &EDO_19),
    scale("24-TET", &EDO_24),
];

const fn scale(name: &'static str, cents: &'static [f32]) -> Scale {
    Scale { name, cents }
}

/// The scale names, for the SCALE choice param.
pub const NAMES: [&str; SCALES.len()] = {
    let mut out = [""; SCALES.len()];
    let mut index = 0;
    while index < SCALES.len() {
        out[index] = SCALES[index].name;
        index += 1;
    }
    out
};

/// Snaps `pitch` (octaves, 0 = C4) to the nearest degree of `scale`
/// rooted `root` semitones above C, in any octave.
#[must_use]
pub fn quantize(pitch: f32, root: f32, scale: &Scale) -> f32 {
    if !pitch.is_finite() {
        return root / 12.0;
    }
    let cents = pitch * 1200.0 - root * 100.0;
    let octave = (cents / 1200.0).floor();
    let within = cents - octave * 1200.0;
    let mut best = 0.0;
    let mut distance = f32::INFINITY;
    // The next octave's root closes the gap above the top degree.
    for &degree in scale.cents.iter().chain(std::iter::once(&1200.0)) {
        let gap = (within - degree).abs();
        if gap < distance {
            distance = gap;
            best = degree;
        }
    }
    (octave * 1200.0 + best + root * 100.0) / 1200.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn find(name: &str) -> &'static Scale {
        SCALES.iter().find(|scale| scale.name == name).unwrap()
    }

    fn semis(pitch: f32) -> f32 {
        pitch * 12.0
    }

    #[test]
    fn every_table_is_sorted_and_inside_the_octave() {
        for scale in &SCALES {
            assert_eq!(scale.cents.first(), Some(&0.0), "{}", scale.name);
            assert!(
                scale.cents.windows(2).all(|pair| pair[0] < pair[1]),
                "{}",
                scale.name
            );
            assert!(scale.cents.iter().all(|&c| c < 1200.0), "{}", scale.name);
        }
        let mut names: Vec<&str> = NAMES.to_vec();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), SCALES.len(), "scale names are unique");
    }

    #[test]
    fn each_degree_maps_to_itself() {
        for scale in &SCALES {
            for octave in [-2.0_f32, 0.0, 1.0] {
                for &degree in scale.cents {
                    let pitch = octave + degree / 1200.0;
                    let out = quantize(pitch, 0.0, scale);
                    assert!((out - pitch).abs() < 1e-5, "{} {degree}", scale.name);
                }
            }
        }
    }

    #[test]
    fn between_degrees_picks_the_nearer() {
        let major = find("Major");
        // C# (100 c) leans toward C or D: 140 c goes to D, 60 c to C.
        assert!((semis(quantize(1.4 / 12.0, 0.0, major)) - 2.0).abs() < 1e-4);
        assert!(semis(quantize(0.6 / 12.0, 0.0, major)).abs() < 1e-4);
        // Above B (1100 c) the next C is nearer than B at 1180 c.
        assert!((semis(quantize(11.8 / 12.0, 0.0, major)) - 12.0).abs() < 1e-4);
    }

    #[test]
    fn root_transposes_the_scale() {
        let major = find("Major");
        // D major has E (4 st) and F# (6 st) but no F: 5.6 st snaps to F#.
        assert!((semis(quantize(5.6 / 12.0, 2.0, major)) - 6.0).abs() < 1e-4);
        assert!((semis(quantize(2.0 / 12.0, 2.0, major)) - 2.0).abs() < 1e-4);
    }

    #[test]
    fn quarter_tones_survive() {
        let rast = find("Maqam Rast");
        let out = quantize(3.4 / 12.0, 0.0, rast);
        assert!((out * 1200.0 - 350.0).abs() < 1e-3, "{}", out * 1200.0);
        let edo24 = find("24-TET");
        let out = quantize(0.55 / 12.0, 0.0, edo24);
        assert!((out * 1200.0 - 50.0).abs() < 1e-3);
    }

    #[test]
    fn negative_pitches_and_octave_wrap() {
        let minor = find("Minor pentatonic");
        // One octave down, a hair under the root: the root.
        let out = quantize(-1.0 - 0.1 / 12.0, 0.0, minor);
        assert!((semis(out) + 12.0).abs() < 1e-4, "{}", semis(out));
        // Just below C4 from the top of the octave below.
        let out = quantize(-1.5 / 12.0, 0.0, minor);
        assert!((semis(out) + 2.0).abs() < 1e-4, "{}", semis(out));
    }

    #[test]
    fn non_finite_input_stays_finite() {
        assert!(quantize(f32::NAN, 3.0, &SCALES[0]).is_finite());
        assert!(quantize(f32::INFINITY, 0.0, &SCALES[0]).is_finite());
    }
}
