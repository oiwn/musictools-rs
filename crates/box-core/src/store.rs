//! Lock-free param and meter storage shared by the UI and the audio thread.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::{ModuleDescriptor, ParamSpec};

/// One `AtomicU32` (f32 bits) per param of one module instance. The UI
/// writes it; the engine copies it into [`ParamValues`] once per block.
#[derive(Debug)]
pub struct ParamStore {
    specs: &'static [ParamSpec],
    values: Box<[AtomicU32]>,
}

impl ParamStore {
    /// A store holding every param's default.
    #[must_use]
    pub fn new(specs: &'static [ParamSpec]) -> Self {
        let values = specs
            .iter()
            .map(|spec| AtomicU32::new(spec.default_value().to_bits()))
            .collect();
        Self { specs, values }
    }

    /// The specs this store holds values for.
    #[must_use]
    pub fn specs(&self) -> &'static [ParamSpec] {
        self.specs
    }

    /// Number of params.
    #[must_use]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// True when the module has no params.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Plain value of param `index`; out-of-range indices read 0.
    #[must_use]
    pub fn get(&self, index: usize) -> f32 {
        self.values
            .get(index)
            .map_or(0.0, |value| f32::from_bits(value.load(Ordering::Relaxed)))
    }

    /// Writes a plain value, sanitized by the spec (non-finite → default).
    pub fn set(&self, index: usize, value: f32) {
        if let (Some(spec), Some(slot)) = (self.specs.get(index), self.values.get(index)) {
            slot.store(spec.sanitize(value).to_bits(), Ordering::Relaxed);
        }
    }

    /// Knob position of param `index`.
    #[must_use]
    pub fn get_normalized(&self, index: usize) -> f32 {
        self.specs
            .get(index)
            .map_or(0.0, |spec| spec.normalize(self.get(index)))
    }

    /// Writes param `index` from a knob position.
    pub fn set_normalized(&self, index: usize, position: f32) {
        if let Some(spec) = self.specs.get(index) {
            self.set(index, spec.denormalize(position));
        }
    }

    /// Index of the param with stable id `id`.
    #[must_use]
    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.specs.iter().position(|spec| spec.id == id)
    }

    /// Copies every value into `out` (realtime-safe); extra slots are left.
    pub fn snapshot_into(&self, out: &mut [f32]) {
        for (dst, src) in out.iter_mut().zip(self.values.iter()) {
            *dst = f32::from_bits(src.load(Ordering::Relaxed));
        }
    }
}

/// One block's param snapshot, read by `Module::process`.
#[derive(Clone, Copy, Debug)]
pub struct ParamValues<'a>(pub &'a [f32]);

impl ParamValues<'_> {
    /// Plain value of param `index`; out-of-range reads 0.
    #[must_use]
    pub fn get(&self, index: usize) -> f32 {
        self.0.get(index).copied().unwrap_or(0.0)
    }

    /// Param `index` as a bool.
    #[must_use]
    pub fn bool(&self, index: usize) -> bool {
        self.get(index) >= 0.5
    }

    /// Param `index` as a choice index.
    #[must_use]
    pub fn choice(&self, index: usize) -> usize {
        self.get(index).max(0.0) as usize
    }
}

/// Values a module publishes for its view (peaks, phases), one atomic each.
#[derive(Debug)]
pub struct Meters {
    values: Box<[AtomicU32]>,
}

impl Meters {
    /// `count` meters reading 0.
    #[must_use]
    pub fn new(count: usize) -> Self {
        Self {
            values: (0..count).map(|_| AtomicU32::new(0)).collect(),
        }
    }

    /// Reads meter `index`; out-of-range reads 0.
    #[must_use]
    pub fn get(&self, index: usize) -> f32 {
        self.values
            .get(index)
            .map_or(0.0, |value| f32::from_bits(value.load(Ordering::Relaxed)))
    }

    /// Writes meter `index` (realtime-safe); out-of-range writes are ignored.
    pub fn set(&self, index: usize, value: f32) {
        if let Some(slot) = self.values.get(index) {
            slot.store(value.to_bits(), Ordering::Relaxed);
        }
    }
}

/// Everything the UI and the audio thread share for one module instance.
#[derive(Debug)]
pub struct Shared {
    pub params: ParamStore,
    pub meters: Meters,
}

impl Shared {
    /// Defaults for a module of this descriptor.
    #[must_use]
    pub fn new(descriptor: &ModuleDescriptor) -> Self {
        Self {
            params: ParamStore::new(descriptor.params),
            meters: Meters::new(descriptor.meters.len()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Taper;

    static SPECS: [ParamSpec; 2] = [
        ParamSpec::float("gain", "Gain", 0.0, 2.0, 1.0, Taper::Linear),
        ParamSpec::bool("on", "On", true),
    ];

    #[test]
    fn store_starts_at_defaults() {
        let store = ParamStore::new(&SPECS);
        assert_eq!(store.get(0), 1.0);
        assert_eq!(store.get(1), 1.0);
    }

    #[test]
    fn store_snapshot_matches_writes() {
        let store = ParamStore::new(&SPECS);
        store.set(0, 0.25);
        store.set(1, 0.0);
        let mut out = [9.0; 2];
        store.snapshot_into(&mut out);
        assert_eq!(out, [0.25, 0.0]);
        assert_eq!(ParamValues(&out).get(0), 0.25);
        assert!(!ParamValues(&out).bool(1));
    }

    #[test]
    fn non_finite_writes_fall_back_to_defaults() {
        let store = ParamStore::new(&SPECS);
        store.set(0, f32::NAN);
        assert_eq!(store.get(0), 1.0);
        store.set(0, 5.0);
        assert_eq!(store.get(0), 2.0);
    }

    #[test]
    fn normalized_access_uses_the_spec() {
        let store = ParamStore::new(&SPECS);
        store.set_normalized(0, 0.25);
        assert!((store.get(0) - 0.5).abs() < 1e-6);
        assert!((store.get_normalized(0) - 0.25).abs() < 1e-6);
        assert_eq!(store.index_of("on"), Some(1));
    }

    #[test]
    fn meters_read_back() {
        let meters = Meters::new(1);
        meters.set(0, 0.5);
        meters.set(3, 1.0);
        assert_eq!(meters.get(0), 0.5);
        assert_eq!(meters.get(3), 0.0);
    }
}
