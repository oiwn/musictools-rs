//! Scope tap — the audio-to-editor handoff the oscilloscope uses: a
//! fixed-capacity ring the audio thread pushes interleaved frames into and
//! the editor drains.
//!
//! The mutex is warmed at construction (locked and released once) because
//! macOS pthread mutexes can allocate lazily on first lock; after the warm-up
//! neither locking nor pushing allocates — at capacity the oldest frame is
//! dropped before each push.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, MutexGuard};

/// Interleaved frame tap with a fixed capacity.
pub struct ScopeTap<T: Copy> {
    frames: Mutex<VecDeque<T>>,
    capacity: usize,
}

impl<T: Copy> ScopeTap<T> {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let tap = Self {
            frames: Mutex::new(VecDeque::with_capacity(capacity)),
            capacity,
        };
        // Warm the lock so the audio thread never pays the lazy allocation.
        drop(tap.lock());
        tap
    }

    /// Append frames; once full the oldest frames are dropped. Allocation-free
    /// after construction.
    pub fn push_frames(&self, frames: &[T]) {
        let mut guard = self.lock();
        for frame in frames {
            if guard.len() == self.capacity {
                guard.pop_front();
            }
            guard.push_back(*frame);
        }
    }

    /// Remove and return all buffered frames (editor side; allocates).
    pub fn drain(&self) -> Vec<T> {
        let mut guard = self.lock();
        guard.drain(..).collect()
    }

    /// Drop all buffered frames.
    pub fn clear(&self) {
        self.lock().clear();
    }

    fn lock(&self) -> MutexGuard<'_, VecDeque<T>> {
        self.frames
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Convenience alias matching how plugins share the tap with their editor.
pub type SharedScopeTap<T> = Arc<ScopeTap<T>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pushes_and_drains_in_order() {
        let tap: ScopeTap<u8> = ScopeTap::new(4);
        tap.push_frames(&[1, 2]);
        tap.push_frames(&[3]);
        assert_eq!(tap.drain(), vec![1, 2, 3]);
        assert!(tap.drain().is_empty());
    }

    #[test]
    fn drops_oldest_frames_once_full() {
        let tap: ScopeTap<u8> = ScopeTap::new(2);
        tap.push_frames(&[1, 2]);
        tap.push_frames(&[3, 4]);
        tap.push_frames(&[5]);
        assert_eq!(tap.drain(), vec![4, 5]);
    }

    #[test]
    fn clear_empties_the_tap() {
        let tap: ScopeTap<u8> = ScopeTap::new(8);
        tap.push_frames(&[9, 9, 9]);
        tap.clear();
        assert!(tap.drain().is_empty());
    }

    #[test]
    fn interleaved_stereo_frames_round_trip() {
        let tap = ScopeTap::new(6);
        tap.push_frames(&[0.5, -0.5, 1.0, -1.0]);
        let drained = tap.drain();
        assert_eq!(drained.as_chunks::<2>().0, &[[0.5_f32, -0.5], [1.0, -1.0]]);
    }
}
