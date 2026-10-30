//! Param bindings — the kit's widgets driving nice-plug parameters through
//! `ParamSetter`. Enabled by the `nice-plug` feature so the core kit stays
//! framework-free.

use crate::{button, knob, select, tabs};
use nice_plug::prelude::*;

/// Knob bound to a `FloatParam`; double-click restores the parameter default.
pub fn param_knob(
    ui: &mut egui::Ui,
    setter: &ParamSetter<'_>,
    param: &FloatParam,
    label: &str,
) -> egui::Response {
    param_knob_with(ui, setter, param, label, knob::KnobParams::default())
}

/// [`param_knob`] with custom knob parameters (size, accent, snap points).
pub fn param_knob_with(
    ui: &mut egui::Ui,
    setter: &ParamSetter<'_>,
    param: &FloatParam,
    label: &str,
    params: knob::KnobParams<'_>,
) -> egui::Response {
    let mut normalized = f64::from(param.modulated_normalized_value());
    // The param's own formatter plus its unit: "-0.10 dB", "1.00 kHz".
    let readout = |normalized: f64| param.normalized_value_to_string(normalized as f32, true);
    let mut knob_params = params;
    knob_params.default = f64::from(param.default_normalized_value());
    let response = knob::knob_with(ui, &mut normalized, label, &readout, &knob_params);
    if response.changed() {
        // Push a plain value, not the raw normalized drag position:
        // `preview_plain` snaps to the param's step size, and
        // `set_parameter` normalizes it exactly — `set_parameter_normalized`
        // skips snapping.
        let plain = param.preview_plain(normalized as f32);
        setter.begin_set_parameter(param);
        setter.set_parameter(param, plain);
        setter.end_set_parameter(param);
    }
    response
}

/// Select bound to an `EnumParam`; the menu lists the enum's variants.
pub fn param_select<T>(
    ui: &mut egui::Ui,
    setter: &ParamSetter<'_>,
    param: &EnumParam<T>,
    id_salt: &str,
) -> egui::Response
where
    T: Enum + PartialEq + Clone + Send + Sync + 'static,
{
    let variants = T::variants();
    let current = param.value();
    let mut index = (0..variants.len())
        .find(|&index| T::from_index(index) == current)
        .unwrap_or(0);
    let id = ui.make_persistent_id(id_salt);
    let response = select::select(ui, id, &mut index, variants);
    if response.changed() {
        setter.begin_set_parameter(param);
        setter.set_parameter(param, T::from_index(index));
        setter.end_set_parameter(param);
    }
    response
}

/// LED toggle bound to a `BoolParam`.
pub fn param_toggle(
    ui: &mut egui::Ui,
    setter: &ParamSetter<'_>,
    param: &BoolParam,
    label: &str,
) -> egui::Response {
    let mut on = param.value();
    let response = button::toggle(ui, &mut on, label);
    if response.clicked() {
        setter.begin_set_parameter(param);
        setter.set_parameter(param, on);
        setter.end_set_parameter(param);
    }
    response
}

/// Segmented tabs bound to an `EnumParam`; the segments are the enum's
/// variants. Switches without any popup.
pub fn param_tabs<T>(
    ui: &mut egui::Ui,
    setter: &ParamSetter<'_>,
    param: &EnumParam<T>,
    _id_salt: &str,
) -> egui::Response
where
    T: Enum + PartialEq + Clone + Send + Sync + 'static,
{
    let variants = T::variants();
    let current = param.value();
    let mut index = (0..variants.len())
        .find(|&index| T::from_index(index) == current)
        .unwrap_or(0);
    let response = tabs::tabs(ui, &mut index, variants);
    if response.changed() {
        setter.begin_set_parameter(param);
        setter.set_parameter(param, T::from_index(index));
        setter.end_set_parameter(param);
    }
    response
}

/// Vertically stacked [`param_tabs`] filling `height` — pass the height of
/// whatever column the stack stands beside (a knob footprint, typically) so
/// the segments use the vertical space and align with it.
pub fn param_tabs_vertical<T>(
    ui: &mut egui::Ui,
    setter: &ParamSetter<'_>,
    param: &EnumParam<T>,
    height: f32,
    _id_salt: &str,
) -> egui::Response
where
    T: Enum + PartialEq + Clone + Send + Sync + 'static,
{
    let variants = T::variants();
    let current = param.value();
    let mut index = (0..variants.len())
        .find(|&index| T::from_index(index) == current)
        .unwrap_or(0);
    let response = tabs::tabs_vertical(ui, &mut index, variants, height);
    if response.changed() {
        setter.begin_set_parameter(param);
        setter.set_parameter(param, T::from_index(index));
        setter.end_set_parameter(param);
    }
    response
}
