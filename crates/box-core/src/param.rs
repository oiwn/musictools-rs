//! Param specs: a stable id, a range with a taper, and display helpers.

use musictools_core::{LogTaper, finite_or};

/// How knob travel (normalized 0..=1) maps onto a float range.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Taper {
    /// Even steps.
    Linear,
    /// `min + (max - min) * t^factor`; factors below 1 spread the low end.
    Skew(f32),
    /// Equal ratios per unit of travel; needs `0 < min < max`.
    Log,
}

/// The value domain of a param. Every kind stores as an `f32`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParamKind {
    Float {
        min: f32,
        max: f32,
        default: f32,
        taper: Taper,
    },
    /// Stored as 0.0 or 1.0.
    Bool { default: bool },
    /// Stored as the option index.
    Choice {
        options: &'static [&'static str],
        default: usize,
    },
}

/// One module param.
#[derive(Clone, Copy, Debug)]
pub struct ParamSpec {
    /// Stable id used by saved patches; never rename it.
    pub id: &'static str,
    pub name: &'static str,
    pub kind: ParamKind,
    /// Unit appended by the default formatter, e.g. "Hz".
    pub unit: &'static str,
    /// Custom formatter (UI thread only, may allocate).
    pub format: Option<fn(f32) -> String>,
    /// Custom parser for typed entry.
    pub parse: Option<fn(&str) -> Option<f32>>,
    /// Glide time the module should smooth this param over.
    pub smoothing_ms: f32,
}

impl ParamSpec {
    /// A float param.
    #[must_use]
    pub const fn float(
        id: &'static str,
        name: &'static str,
        min: f32,
        max: f32,
        default: f32,
        taper: Taper,
    ) -> Self {
        Self::new(
            id,
            name,
            ParamKind::Float {
                min,
                max,
                default,
                taper,
            },
        )
    }

    /// An on/off param.
    #[must_use]
    pub const fn bool(id: &'static str, name: &'static str, default: bool) -> Self {
        Self::new(id, name, ParamKind::Bool { default })
    }

    /// A choice among named options.
    #[must_use]
    pub const fn choice(
        id: &'static str,
        name: &'static str,
        options: &'static [&'static str],
        default: usize,
    ) -> Self {
        Self::new(id, name, ParamKind::Choice { options, default })
    }

    const fn new(id: &'static str, name: &'static str, kind: ParamKind) -> Self {
        Self {
            id,
            name,
            kind,
            unit: "",
            format: None,
            parse: None,
            smoothing_ms: 0.0,
        }
    }

    /// Sets the unit shown by the default formatter.
    #[must_use]
    pub const fn unit(mut self, unit: &'static str) -> Self {
        self.unit = unit;
        self
    }

    /// Sets the smoothing time in milliseconds.
    #[must_use]
    pub const fn smoothing(mut self, ms: f32) -> Self {
        self.smoothing_ms = ms;
        self
    }

    /// Sets a custom formatter.
    #[must_use]
    pub const fn formatter(mut self, format: fn(f32) -> String) -> Self {
        self.format = Some(format);
        self
    }

    /// Sets a custom parser.
    #[must_use]
    pub const fn parser(mut self, parse: fn(&str) -> Option<f32>) -> Self {
        self.parse = Some(parse);
        self
    }

    /// The default plain value.
    #[must_use]
    pub fn default_value(&self) -> f32 {
        match self.kind {
            ParamKind::Float { default, .. } => default,
            ParamKind::Bool { default } => f32::from(u8::from(default)),
            ParamKind::Choice { default, .. } => default as f32,
        }
    }

    /// Lowest and highest plain values.
    #[must_use]
    pub fn range(&self) -> (f32, f32) {
        match self.kind {
            ParamKind::Float { min, max, .. } => (min, max),
            ParamKind::Bool { .. } => (0.0, 1.0),
            ParamKind::Choice { options, .. } => (0.0, options.len().saturating_sub(1) as f32),
        }
    }

    /// Clamps a plain value into range, snapping bools and choices; a
    /// non-finite value falls back to the default.
    #[must_use]
    pub fn sanitize(&self, value: f32) -> f32 {
        if !value.is_finite() {
            return self.default_value();
        }
        let (min, max) = self.range();
        let value = value.clamp(min, max);
        match self.kind {
            ParamKind::Float { .. } => value,
            ParamKind::Bool { .. } => f32::from(u8::from(value >= 0.5)),
            ParamKind::Choice { .. } => value.round(),
        }
    }

    /// Knob position (0..=1) of a plain value.
    #[must_use]
    pub fn normalize(&self, value: f32) -> f32 {
        let value = self.sanitize(value);
        let (min, max) = self.range();
        if max <= min {
            return 0.0;
        }
        let linear = (value - min) / (max - min);
        let position = match self.kind {
            ParamKind::Float {
                taper: Taper::Skew(factor),
                ..
            } if factor > 0.0 => linear.powf(1.0 / factor),
            ParamKind::Float {
                taper: Taper::Log, ..
            } => LogTaper::new(min, max).to_position(value),
            _ => linear,
        };
        position.clamp(0.0, 1.0)
    }

    /// Plain value at a knob position (0..=1); out-of-range and non-finite
    /// positions clamp.
    #[must_use]
    pub fn denormalize(&self, position: f32) -> f32 {
        let position = finite_or(position, 0.0).clamp(0.0, 1.0);
        let (min, max) = self.range();
        let value = match self.kind {
            ParamKind::Float {
                taper: Taper::Skew(factor),
                ..
            } if factor > 0.0 => min + (max - min) * position.powf(factor),
            ParamKind::Float {
                taper: Taper::Log, ..
            } => LogTaper::new(min, max).to_value(position),
            _ => min + (max - min) * position,
        };
        self.sanitize(value)
    }

    /// Display text for a plain value (UI thread only).
    #[must_use]
    pub fn display(&self, value: f32) -> String {
        if let Some(format) = self.format {
            return format(value);
        }
        match self.kind {
            ParamKind::Float { .. } => {
                let digits = if value.abs() >= 100.0 {
                    0
                } else if value.abs() >= 10.0 {
                    1
                } else {
                    2
                };
                if self.unit.is_empty() {
                    format!("{value:.digits$}")
                } else {
                    format!("{value:.digits$} {}", self.unit)
                }
            }
            ParamKind::Bool { .. } => if value >= 0.5 { "On" } else { "Off" }.to_owned(),
            ParamKind::Choice { options, .. } => options
                .get(value.round().max(0.0) as usize)
                .copied()
                .unwrap_or("?")
                .to_owned(),
        }
    }

    /// Parses typed text into a sanitized plain value.
    #[must_use]
    pub fn parse_text(&self, text: &str) -> Option<f32> {
        if let Some(parse) = self.parse {
            return parse(text).map(|value| self.sanitize(value));
        }
        let trimmed = text.trim().trim_end_matches(self.unit).trim();
        trimmed
            .parse::<f32>()
            .ok()
            .map(|value| self.sanitize(value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: ParamSpec = ParamSpec::float("freq", "Freq", 20.0, 20_000.0, 440.0, Taper::Log);
    const SKEW: ParamSpec = ParamSpec::float("time", "Time", 0.0, 10.0, 1.0, Taper::Skew(0.3));
    const LIN: ParamSpec = ParamSpec::float("level", "Level", 0.0, 1.0, 0.5, Taper::Linear);

    #[test]
    fn tapers_invert() {
        for spec in [LOG, SKEW, LIN] {
            for step in 0..=20 {
                let position = step as f32 / 20.0;
                let back = spec.normalize(spec.denormalize(position));
                assert!(
                    (back - position).abs() < 1e-4,
                    "{}: {position} -> {back}",
                    spec.id
                );
            }
        }
    }

    #[test]
    fn log_taper_middle_is_geometric_mean() {
        let middle = LOG.denormalize(0.5);
        assert!((middle - 632.455_5).abs() < 0.1, "{middle}");
    }

    #[test]
    fn non_finite_values_use_fallback() {
        assert_eq!(LOG.sanitize(f32::NAN), 440.0);
        assert_eq!(LOG.sanitize(f32::INFINITY), 440.0);
        assert_eq!(LOG.denormalize(f32::NAN), 20.0);
    }

    #[test]
    fn bools_and_choices_snap() {
        let toggle = ParamSpec::bool("on", "On", false);
        assert_eq!(toggle.sanitize(0.7), 1.0);
        let mode = ParamSpec::choice("mode", "Mode", &["A", "B", "C"], 1);
        assert_eq!(mode.default_value(), 1.0);
        assert_eq!(mode.sanitize(1.6), 2.0);
        assert_eq!(mode.display(2.0), "C");
    }

    #[test]
    fn display_and_parse_round_trip() {
        let freq = LOG.unit("Hz");
        assert_eq!(freq.display(440.0), "440 Hz");
        assert_eq!(freq.parse_text("440 Hz"), Some(440.0));
        assert_eq!(freq.parse_text("1e9"), Some(20_000.0));
        assert_eq!(freq.parse_text("abc"), None);
    }
}
