use std::collections::VecDeque;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_honors_capacity() {
        let mut b = ParticleBuffer::new(2, false);
        assert!(b.push(ParticleSpawn::default()));
        assert!(b.push(ParticleSpawn::default()));
        assert!(!b.push(ParticleSpawn::default()));   // over capacity
        assert_eq!(b.count(), 2);
    }

    #[test]
    fn swap_remove_compacts() {
        let mut b = ParticleBuffer::new(4, false);
        let mut s = ParticleSpawn::default();
        s.lifetime = 1.0;
        for _ in 0..4 { b.push(s.clone()); }
        b.age[0] = 0.5;
        b.age[1] = 1.5;   // dead
        b.age[2] = 0.3;
        b.age[3] = 2.0;   // dead
        b.compact_dead(|_, _, _| {});
        assert_eq!(b.count(), 2);
    }

    #[test]
    fn trail_allocated_only_when_ribbon() {
        let no_trail = ParticleBuffer::new(4, false);
        assert!(no_trail.trail.is_none());
        let with_trail = ParticleBuffer::new(4, true);
        assert!(with_trail.trail.is_some());
    }

    #[test]
    fn compact_dead_invokes_callback() {
        let mut b = ParticleBuffer::new(4, false);
        let mut s = ParticleSpawn::default();
        s.lifetime = 1.0;
        for _ in 0..4 { b.push(s.clone()); }
        b.age[1] = 2.0;
        b.age[3] = 2.0;
        let mut killed: Vec<usize> = vec![];
        b.compact_dead(|pos, vel, color| {
            killed.push(0);
            let _ = (pos, vel, color);
        });
        assert_eq!(killed.len(), 2);
    }
}

#[derive(Clone, Debug, Default)]
pub struct ParticleSpawn {
    pub position:  [f32; 3],
    pub velocity:  [f32; 3],
    pub color:     [f32; 4],
    pub size:      f32,
    pub rotation:  f32,
    pub lifetime:  f32,
    pub seed:      f32,
}

pub struct ParticleBuffer {
    pub position:       Vec<[f32; 3]>,
    pub velocity:       Vec<[f32; 3]>,
    pub color:          Vec<[f32; 4]>,
    pub size:           Vec<f32>,
    pub rotation:       Vec<f32>,
    pub age:            Vec<f32>,
    pub lifetime:       Vec<f32>,
    pub seed:           Vec<f32>,
    pub flipbook_frame: Vec<f32>,
    pub trail:          Option<Vec<VecDeque<[f32; 3]>>>,
    capacity:           usize,
}

impl ParticleBuffer {
    pub fn new(capacity: usize, with_trails: bool) -> Self {
        ParticleBuffer {
            position:       Vec::with_capacity(capacity),
            velocity:       Vec::with_capacity(capacity),
            color:          Vec::with_capacity(capacity),
            size:           Vec::with_capacity(capacity),
            rotation:       Vec::with_capacity(capacity),
            age:            Vec::with_capacity(capacity),
            lifetime:       Vec::with_capacity(capacity),
            seed:           Vec::with_capacity(capacity),
            flipbook_frame: Vec::with_capacity(capacity),
            trail:          if with_trails { Some(Vec::with_capacity(capacity)) } else { None },
            capacity,
        }
    }

    pub fn count(&self) -> usize { self.position.len() }
    pub fn capacity(&self) -> usize { self.capacity }
    pub fn is_full(&self) -> bool { self.count() >= self.capacity }

    pub fn push(&mut self, s: ParticleSpawn) -> bool {
        if self.is_full() { return false; }
        self.position.push(s.position);
        self.velocity.push(s.velocity);
        self.color.push(s.color);
        self.size.push(s.size);
        self.rotation.push(s.rotation);
        self.age.push(0.0);
        self.lifetime.push(s.lifetime);
        self.seed.push(s.seed);
        self.flipbook_frame.push(0.0);
        if let Some(t) = &mut self.trail { t.push(VecDeque::new()); }
        true
    }

    /// Remove all particles whose age >= lifetime, in-place, compacting via swap_remove.
    /// Callback invoked for each killed particle with (position, velocity, color).
    pub fn compact_dead(&mut self, mut on_kill: impl FnMut([f32; 3], [f32; 3], [f32; 4])) {
        let mut i = 0;
        while i < self.position.len() {
            if self.age[i] >= self.lifetime[i] {
                on_kill(self.position[i], self.velocity[i], self.color[i]);
                self.position.swap_remove(i);
                self.velocity.swap_remove(i);
                self.color.swap_remove(i);
                self.size.swap_remove(i);
                self.rotation.swap_remove(i);
                self.age.swap_remove(i);
                self.lifetime.swap_remove(i);
                self.seed.swap_remove(i);
                self.flipbook_frame.swap_remove(i);
                if let Some(t) = &mut self.trail { t.swap_remove(i); }
                // don't advance i — new particle at this slot
            } else {
                i += 1;
            }
        }
    }
}
