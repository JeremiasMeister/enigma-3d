use std::vec::Vec;
use nalgebra::{Matrix4, UnitQuaternion, Vector3, Quaternion};
use serde::{Deserialize, Serialize};
use crate::logging::{EnigmaError};
use crate::smart_format;

pub(crate) const MAX_BONES: usize = 128;

#[derive(Clone, Debug)]
pub struct Bone {
    pub name: String,
    pub id: usize,
    pub node_index: usize,
    pub parent_id: Option<usize>,
    pub inverse_bind_pose: Matrix4<f32>,
}

#[derive(Clone, Debug)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
}


impl Skeleton {
    pub fn validate(&self) -> Result<(), EnigmaError> {
        for bone in &self.bones {
            if let Some(parent_id) = bone.parent_id {
                if parent_id >= self.bones.len() {
                    return Err(EnigmaError::new(
                        Some(
                            smart_format!(
                                "Invalid parent ID {} for bone {} with id {}. There are only {} bones in the skeleton.",
                                parent_id,
                                bone.name,
                                bone.id,
                                self.bones.len()
                            )
                                .as_str(),
                        ),
                        true,
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn try_fix(&mut self) -> Result<(), EnigmaError> {
        let len = self.bones.len();
        for bone in &mut self.bones {
            if let Some(parent_id) = bone.parent_id {
                if parent_id >= len {
                    bone.parent_id = None;
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct AnimationKeyframe<T> {
    pub time: f32,
    pub value: T,
}

#[derive(Clone)]
pub struct AnimationChannel {
    pub bone_id: usize,
    pub translations: Vec<AnimationKeyframe<[f32; 3]>>,
    pub rotations: Vec<AnimationKeyframe<[f32; 4]>>,
    pub scales: Vec<AnimationKeyframe<[f32; 3]>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnimationState {
    pub name: String,
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
}

#[derive(Clone)]
pub struct Animation {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<AnimationChannel>,
}

fn find_keyframes<T>(
    keyframes: &[AnimationKeyframe<T>],
    time: f32,
) -> (&AnimationKeyframe<T>, &AnimationKeyframe<T>) {
    // If time is before the first keyframe, return the first keyframe for both prev and next
    if time <= keyframes[0].time {
        return (&keyframes[0], &keyframes[0]);
    }

    // If time is after the last keyframe, return the last keyframe for both prev and next
    if time >= keyframes[keyframes.len() - 1].time {
        let last = &keyframes[keyframes.len() - 1];
        return (last, last);
    }

    // Find keyframes surrounding the current time
    for i in 0..keyframes.len() - 1 {
        if time >= keyframes[i].time && time <= keyframes[i + 1].time {
            return (&keyframes[i], &keyframes[i + 1]);
        }
    }

    // Default case (should not reach here)
    (&keyframes[0], &keyframes[0])
}

pub fn interpolate_keyframes(animation: &Animation, bone_id: usize, time: f32) -> Matrix4<f32> {
    if let Some(channel) = animation.channels.iter().find(|c| c.bone_id == bone_id) {
        // Interpolate Translation
        let translation_matrix = interpolate_transform(
            &channel.translations,
            time,
            Matrix4::identity(),
            |prev, next, t| interpolate_translation(&prev.value, &next.value, t),
        );

        // Interpolate Rotation
        let rotation_matrix = interpolate_transform(
            &channel.rotations,
            time,
            Matrix4::identity(),
            |prev, next, t| interpolate_rotation(&prev.value, &next.value, t),
        );

        // Interpolate Scale
        let scale_matrix = interpolate_transform(
            &channel.scales,
            time,
            Matrix4::identity(),
            |prev, next, t| interpolate_scale(&prev.value, &next.value, t),
        );

        // Combine the transformations
        translation_matrix * rotation_matrix * scale_matrix
    } else {
        Matrix4::identity()
    }
}

fn interpolate_transform<T, F>(
    keyframes: &[AnimationKeyframe<T>],
    time: f32,
    default: Matrix4<f32>,
    interpolate_fn: F,
) -> Matrix4<f32>
    where
        F: Fn(&AnimationKeyframe<T>, &AnimationKeyframe<T>, f32) -> Matrix4<f32>,
{
    if keyframes.is_empty() {
        return default;
    }

    let (prev, next) = find_keyframes(keyframes, time);
    let t = if next.time != prev.time {
        (time - prev.time) / (next.time - prev.time)
    } else {
        0.0
    };
    interpolate_fn(prev, next, t)
}

fn interpolate_translation(prev: &[f32; 3], next: &[f32; 3], t: f32) -> Matrix4<f32> {
    let start = Vector3::from(*prev);
    let end = Vector3::from(*next);
    let interpolated = start.lerp(&end, t);
    Matrix4::new_translation(&interpolated)
}

fn interpolate_rotation(prev: &[f32; 4], next: &[f32; 4], t: f32) -> Matrix4<f32> {
    let start = UnitQuaternion::from_quaternion(Quaternion::new(prev[3], prev[0], prev[1], prev[2]));
    let end = UnitQuaternion::from_quaternion(Quaternion::new(next[3], next[0], next[1], next[2]));
    start.slerp(&end, t).to_homogeneous()
}

fn interpolate_scale(prev: &[f32; 3], next: &[f32; 3], t: f32) -> Matrix4<f32> {
    let start = Vector3::from(*prev);
    let end = Vector3::from(*next);
    let interpolated = start.lerp(&end, t);
    Matrix4::new_nonuniform_scaling(&interpolated)
}
