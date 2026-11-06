//! Compact readouts for param specs: short enough to sit under a small
//! knob. Plain functions, so descriptors can use them without egui.

/// Frequency: `440 Hz`, `12.5k`, `2.00 Hz`.
#[must_use]
pub fn short_hz(value: f32) -> String {
    if value >= 1000.0 {
        format!("{:.1}k", value / 1000.0)
    } else if value >= 10.0 {
        format!("{value:.0} Hz")
    } else {
        format!("{value:.2} Hz")
    }
}

/// Time in milliseconds or seconds.
#[must_use]
pub fn short_ms(value: f32) -> String {
    if value >= 1000.0 {
        format!("{:.2} s", value / 1000.0)
    } else if value >= 10.0 {
        format!("{value:.0} ms")
    } else {
        format!("{value:.1} ms")
    }
}

/// A percentage of a 0..=1 value.
#[must_use]
pub fn short_percent(value: f32) -> String {
    format!("{:.0}%", value * 100.0)
}

/// A linear gain in decibels.
#[must_use]
pub fn short_db(value: f32) -> String {
    if value <= 0.000_01 {
        "-inf".to_owned()
    } else {
        format!("{:.1} dB", 20.0 * value.log10())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_formats_stay_short() {
        assert_eq!(short_hz(440.0), "440 Hz");
        assert_eq!(short_hz(12_500.0), "12.5k");
        assert_eq!(short_ms(1500.0), "1.50 s");
        assert_eq!(short_ms(5.0), "5.0 ms");
        assert_eq!(short_percent(0.5), "50%");
        assert_eq!(short_db(1.0), "0.0 dB");
        assert_eq!(short_db(0.5), "-6.0 dB");
        assert_eq!(short_db(0.0), "-inf");
    }
}
