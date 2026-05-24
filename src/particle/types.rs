use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Range<T> {
    pub min: T,
    pub max: T,
}

impl<T: Copy> Range<T> {
    pub fn new(min: T, max: T) -> Self {
        Range { min, max }
    }
}

impl Range<f32> {
    pub fn sample(&self, rng: &mut impl rand::Rng) -> f32 {
        if self.min == self.max { self.min } else { rng.gen_range(self.min..=self.max) }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct BurstSpec {
    pub time: f32,
    pub count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn range_serde_roundtrip() {
        let r: Range<f32> = Range { min: 0.5, max: 2.5 };
        let json = serde_json::to_string(&r).unwrap();
        let back: Range<f32> = serde_json::from_str(&json).unwrap();
        assert_eq!(back.min, 0.5);
        assert_eq!(back.max, 2.5);
    }

    #[test]
    fn burst_serde_roundtrip() {
        let b = BurstSpec { time: 1.2, count: 8 };
        let json = serde_json::to_string(&b).unwrap();
        let back: BurstSpec = serde_json::from_str(&json).unwrap();
        assert_eq!(back.time, 1.2);
        assert_eq!(back.count, 8);
    }
}
