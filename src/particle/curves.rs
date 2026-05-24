use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Curve<T: Copy> {
    pub keys: Vec<(f32, T)>,
}

impl<T: Copy + Default> Curve<T> {
    pub fn new(keys: Vec<(f32, T)>) -> Self {
        Curve { keys }
    }
}

impl Curve<f32> {
    pub fn eval(&self, t: f32) -> f32 {
        if self.keys.is_empty() {
            return 0.0;
        }
        if t <= self.keys[0].0 {
            return self.keys[0].1;
        }
        if t >= self.keys[self.keys.len() - 1].0 {
            return self.keys[self.keys.len() - 1].1;
        }
        for i in 0..self.keys.len() - 1 {
            let (t0, v0) = self.keys[i];
            let (t1, v1) = self.keys[i + 1];
            if t >= t0 && t <= t1 {
                let f = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };
                return v0 + (v1 - v0) * f;
            }
        }
        self.keys[self.keys.len() - 1].1
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Gradient<T: Copy> {
    pub keys: Vec<(f32, T)>,
}

impl<T: Copy + Default> Gradient<T> {
    pub fn new(keys: Vec<(f32, T)>) -> Self {
        Gradient { keys }
    }
}

impl Gradient<[f32; 4]> {
    pub fn eval(&self, t: f32) -> [f32; 4] {
        if self.keys.is_empty() {
            return [0.0; 4];
        }
        if t <= self.keys[0].0 {
            return self.keys[0].1;
        }
        if t >= self.keys[self.keys.len() - 1].0 {
            return self.keys[self.keys.len() - 1].1;
        }
        for i in 0..self.keys.len() - 1 {
            let (t0, v0) = self.keys[i];
            let (t1, v1) = self.keys[i + 1];
            if t >= t0 && t <= t1 {
                let f = if t1 > t0 { (t - t0) / (t1 - t0) } else { 0.0 };
                return [
                    v0[0] + (v1[0] - v0[0]) * f,
                    v0[1] + (v1[1] - v0[1]) * f,
                    v0[2] + (v1[2] - v0[2]) * f,
                    v0[3] + (v1[3] - v0[3]) * f,
                ];
            }
        }
        self.keys[self.keys.len() - 1].1
    }
}

pub fn validate_keyframes<T: Copy>(keys: &[(f32, T)]) -> Result<(), String> {
    if keys.is_empty() {
        return Ok(());
    }
    if keys[0].0 < 0.0 || keys[0].0 > 1.0 {
        return Err(format!("keyframe time {} out of [0,1]", keys[0].0));
    }
    for i in 1..keys.len() {
        if keys[i].0 < keys[i - 1].0 {
            return Err(format!("keyframes unsorted at index {}", i));
        }
        if keys[i].0 > 1.0 {
            return Err(format!("keyframe time {} out of [0,1]", keys[i].0));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_keyframe_exact() {
        let c = Curve::new(vec![(0.0, 1.0), (0.5, 3.0), (1.0, 2.0)]);
        assert_eq!(c.eval(0.0), 1.0);
        assert_eq!(c.eval(0.5), 3.0);
        assert_eq!(c.eval(1.0), 2.0);
    }

    #[test]
    fn curve_interpolation() {
        let c = Curve::new(vec![(0.0, 0.0), (1.0, 10.0)]);
        assert!((c.eval(0.25) - 2.5).abs() < 1e-5);
        assert!((c.eval(0.75) - 7.5).abs() < 1e-5);
    }

    #[test]
    fn curve_clamps_out_of_range() {
        let c = Curve::new(vec![(0.0, 5.0), (1.0, 10.0)]);
        assert_eq!(c.eval(-0.5), 5.0);
        assert_eq!(c.eval(1.5), 10.0);
    }

    #[test]
    fn curve_empty_returns_default() {
        let c: Curve<f32> = Curve::new(vec![]);
        assert_eq!(c.eval(0.5), 0.0);
    }

    #[test]
    fn gradient_vec4_midpoint() {
        let g = Gradient::new(vec![
            (0.0, [0.0, 0.0, 0.0, 0.0]),
            (1.0, [1.0, 1.0, 1.0, 1.0]),
        ]);
        let v = g.eval(0.5);
        assert!((v[0] - 0.5).abs() < 1e-5);
        assert!((v[3] - 0.5).abs() < 1e-5);
    }
}
