//! Parameter text and taper helpers shared by the plugin wrappers.
//!
//! The wrappers store values in a framework's parameter type; these turn them
//! into the strings a host or editor shows and back from what a user types.

/// Parse "1000", "1000 Hz", or "1.5 kHz" into hertz.
#[must_use]
pub fn parse_hz(text: &str) -> Option<f32> {
    let lower = text.trim().to_ascii_lowercase();
    let value = match lower.strip_suffix("khz") {
        Some(khz) => khz.trim().parse::<f32>().ok().map(|value| value * 1_000.0),
        None => lower.trim_end_matches("hz").trim().parse::<f32>().ok(),
    }?;
    value.is_finite().then_some(value)
}

/// Show a `0..1` value as a whole percentage, without the unit ("55").
///
/// Pair with a `" %"` unit on the parameter so the unit is printed once.
#[must_use]
pub fn format_percent(value: f32) -> String {
    format!("{:.0}", crate::finite_or(value, 0.0) * 100.0)
}

/// Parse "55", "55%", or "55 %" into `0.55`.
#[must_use]
pub fn parse_percent(text: &str) -> Option<f32> {
    let percent = text
        .trim()
        .trim_end_matches('%')
        .trim()
        .parse::<f32>()
        .ok()?;
    percent.is_finite().then_some(percent / 100.0)
}

/// An exact logarithmic taper between two positive frequencies.
///
/// A knob position in `0..=1` maps to `min * (max / min)^position`, so equal
/// knob travel is an equal frequency ratio. Frameworks without a log range
/// store the position and convert at the edges with this.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogTaper {
    min: f32,
    max: f32,
}

impl LogTaper {
    /// A taper over `min..=max`. Both ends must be positive and `min < max`.
    #[must_use]
    pub const fn new(min: f32, max: f32) -> Self {
        assert!(min > 0.0 && max > min, "a log taper needs 0 < min < max");
        Self { min, max }
    }

    /// Lowest value, at position 0.
    #[must_use]
    pub const fn min(self) -> f32 {
        self.min
    }

    /// Highest value, at position 1.
    #[must_use]
    pub const fn max(self) -> f32 {
        self.max
    }

    /// Value at a knob `position`; out-of-range and non-finite positions clamp.
    #[must_use]
    pub fn to_value(self, position: f32) -> f32 {
        let position = crate::finite_or(position, 0.0).clamp(0.0, 1.0);
        (self.min * (self.max / self.min).powf(position)).clamp(self.min, self.max)
    }

    /// Knob position of `value`; values outside the range, or non-finite,
    /// clamp to the nearer end (NaN to the bottom).
    #[must_use]
    pub fn to_position(self, value: f32) -> f32 {
        if value.is_nan() || value <= self.min {
            return 0.0;
        }
        if value >= self.max {
            return 1.0;
        }
        ((value / self.min).ln() / (self.max / self.min).ln()).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const AUDIO: LogTaper = LogTaper::new(20.0, 20_000.0);

    #[test]
    fn hz_text_parses_with_or_without_units() {
        assert_eq!(parse_hz("1000"), Some(1000.0));
        assert_eq!(parse_hz(" 1000 Hz "), Some(1000.0));
        assert_eq!(parse_hz("1.5 kHz"), Some(1500.0));
        assert_eq!(parse_hz("1.5khz"), Some(1500.0));
        assert_eq!(parse_hz("loud"), None);
        assert_eq!(parse_hz("inf"), None);
    }

    #[test]
    fn percentages_round_trip_through_text() {
        assert_eq!(format_percent(0.55), "55");
        assert_eq!(format_percent(1.0), "100");
        assert_eq!(format_percent(f32::NAN), "0");
        assert_eq!(parse_percent("55"), Some(0.55));
        assert_eq!(parse_percent("55%"), Some(0.55));
        assert_eq!(parse_percent(" 55 % "), Some(0.55));
        assert_eq!(parse_percent("half"), None);
    }

    #[test]
    fn the_log_taper_spends_equal_travel_on_equal_ratios() {
        assert_eq!(AUDIO.to_value(0.0), 20.0);
        assert!((AUDIO.to_value(1.0) - 20_000.0).abs() < 0.5);
        // Three decades: each third of travel is one decade.
        assert!((AUDIO.to_value(1.0 / 3.0) - 200.0).abs() < 0.05);
        assert!((AUDIO.to_value(0.5) - 632.46).abs() < 0.05);
        for hz in [20.0_f32, 63.0, 440.0, 1_000.0, 12_345.0] {
            let back = AUDIO.to_value(AUDIO.to_position(hz));
            assert!((back - hz).abs() / hz < 1.0e-4, "{hz} came back as {back}");
        }
    }

    #[test]
    fn the_log_taper_clamps_malformed_input() {
        assert_eq!(AUDIO.to_position(f32::NAN), 0.0);
        assert_eq!(AUDIO.to_position(-5.0), 0.0);
        assert_eq!(AUDIO.to_position(f32::INFINITY), 1.0);
        assert_eq!(AUDIO.to_value(f32::NAN), 20.0);
        assert_eq!(AUDIO.to_value(-1.0), 20.0);
        assert_eq!(AUDIO.to_value(7.0), AUDIO.max());
    }
}
