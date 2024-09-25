use std::vec::Vec;
use nalgebra::{Matrix4, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};
use crate::logging::EnigmaError;
use crate::smart_format;

pub(crate) const MAX_BONES: usize = 128;

#[derive(Clone, Debug)]
pub struct Bone {
    pub name: String,
    pub id: usize,
    pub parent_id: Option<usize>,
    pub inverse_bind_pose: Matrix4<f32>
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BoneSerializer {
    pub name: String,
    pub id: usize,
    pub parent_id: Option<usize>,
    pub inverse_bind_pose: [[f32;4];4]
}

impl Bone {
    pub fn to_serializer(&self) -> BoneSerializer {
        BoneSerializer {
            name: self.name.clone(),
            id: self.id,
            parent_id: self.parent_id,
            inverse_bind_pose: self.inverse_bind_pose.into()
        }
    }

    pub fn from_serializer(serializer: BoneSerializer) -> Self{
        Self {
            name: serializer.name,
            id: serializer.id,
            parent_id: serializer.parent_id,
            inverse_bind_pose: Matrix4::from(serializer.inverse_bind_pose)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Skeleton {
    pub bones: Vec<Bone>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct SkeletonSerializer {
    pub bones: Vec<BoneSerializer>,
}

impl Skeleton {
    pub fn to_serializer(&self) -> SkeletonSerializer {
        SkeletonSerializer {
            bones: self.bones.iter().map(|x| x.to_serializer()).collect()
        }
    }

    pub fn from_serializer(serializer: SkeletonSerializer) -> Self {
        let mut bones = Vec::new();
        for s in serializer.bones {
            let bone = Bone::from_serializer(s);
            bones.push(bone);
        }
        Self {
            bones
        }
    }
    pub fn validate(&self) -> Result<(), EnigmaError> {
        for bone in self.bones.iter() {
            if let Some(parent_id) = bone.parent_id {
                if parent_id >= self.bones.len() {
                    return Err(EnigmaError::new(Some(smart_format!("Invalid parent ID {} for bone {} with id {}. There are only {} bones in the skeleton.", parent_id, bone.name, bone.id, self.bones.len()).as_str()), true))
                }
            }
        }
        Ok(())
    }

    pub fn try_fix(&mut self) -> Result<(), EnigmaError> {
        let len = self.bones.len();
        for bone in self.bones.iter_mut() {
            if let Some(parent_id) = bone.parent_id {
                if parent_id >= len {
                    bone.parent_id = None;
                }
            }
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum AnimationTransform {
    Translation([f32; 3]),
    Rotation([f32;4]),
    Scale([f32;3])
}

#[derive(Clone)]
pub struct AnimationKeyframe{
    pub time: f32,
    pub transform: AnimationTransform,
}

impl AnimationKeyframe {

    pub fn to_serializer(&self) -> AnimationKeyframeSerializer {
        AnimationKeyframeSerializer {
            time: self.time.clone(),
            transform: self.transform.clone()
        }
    }

    pub fn from_serializer(serializer: AnimationKeyframeSerializer) -> Self {
        Self {
            time: serializer.time,
            transform: serializer.transform
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimationKeyframeSerializer {
    pub time: f32,
    pub transform: AnimationTransform,
}

#[derive(Clone)]
pub struct AnimationChannel {
    pub bone_id: usize,
    pub keyframes: Vec<AnimationKeyframe>,
}
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AnimationState {
    pub name: String,
    pub time: f32,
    pub speed: f32,
    pub looping: bool,
}

impl AnimationChannel {
    pub fn to_serializer(&self) -> AnimationChannelSerializer {
        AnimationChannelSerializer {
            bone_id: self.bone_id.clone(),
            keyframes: self.keyframes.iter().map(|x| x.to_serializer()).collect()
        }
    }

    pub fn from_serializer(serializer: AnimationChannelSerializer) -> Self {
        let mut keyframes = Vec::new();
        for s in serializer.keyframes {
            let keyframe = AnimationKeyframe::from_serializer(s);
            keyframes.push(keyframe);
        }
        Self {
            bone_id: serializer.bone_id,
            keyframes
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimationChannelSerializer {
    pub bone_id: usize,
    pub keyframes: Vec<AnimationKeyframeSerializer>,
}

#[derive(Clone)]
pub struct Animation {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<AnimationChannel>,
}

impl Animation {
    pub fn to_serializer(&self) -> AnimationSerializer {
        AnimationSerializer {
            name: self.name.clone(),
            duration: self.duration.clone(),
            channels: self.channels.iter().map(|x| x.to_serializer()).collect()
        }
    }

    pub fn from_serializer(serializer: AnimationSerializer) -> Self {
        let mut channels = Vec::new();
        for s in serializer.channels {
            let channel = AnimationChannel::from_serializer(s);
            channels.push(channel);
        }
        Self {
            name: serializer.name,
            duration: serializer.duration,
            channels
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AnimationSerializer {
    pub name: String,
    pub duration: f32,
    pub channels: Vec<AnimationChannelSerializer>,
}

fn find_keyframes(channel: &AnimationChannel, time: f32) -> (&AnimationKeyframe, &AnimationKeyframe) {
    let mut prev = &channel.keyframes[0];
    let mut next = prev;

    for keyframe in &channel.keyframes {
        if keyframe.time > time {
            next = keyframe;
            break;
        }
        prev = keyframe;
    }

    (prev, next)
}

pub fn interpolate_keyframes(animation: &Animation, bone_id: usize, time: f32) -> Matrix4<f32> {
    if let Some(channel) = animation.channels.iter().find(|c| c.bone_id == bone_id) {
        let (prev_keyframe, next_keyframe) = find_keyframes(channel, time);

        let t = if next_keyframe.time != prev_keyframe.time {
            (time - prev_keyframe.time) / (next_keyframe.time - prev_keyframe.time)
        } else {
            0.0
        };

        let translation = interpolate_translation(&prev_keyframe.transform, &next_keyframe.transform, t);
        let rotation = interpolate_rotation(&prev_keyframe.transform, &next_keyframe.transform, t);
        let scale = interpolate_scale(&prev_keyframe.transform, &next_keyframe.transform, t);

        // Combine the transformations
        translation * rotation * scale
    } else {
        Matrix4::identity()
    }
}

fn interpolate_translation(prev: &AnimationTransform, next: &AnimationTransform, t: f32) -> Matrix4<f32> {
    match (prev, next) {
        (AnimationTransform::Translation(p), AnimationTransform::Translation(n)) => {
            let start = Vector3::from(*p);
            let end = Vector3::from(*n);
            let interpolated = start.lerp(&end, t);
            Matrix4::new_translation(&interpolated)
        },
        _ => Matrix4::identity(),
    }
}

fn interpolate_rotation(prev: &AnimationTransform, next: &AnimationTransform, t: f32) -> Matrix4<f32> {
    match (prev, next) {
        (AnimationTransform::Rotation(p), AnimationTransform::Rotation(n)) => {
            let start = UnitQuaternion::from_quaternion(nalgebra::Quaternion::new(p[3], p[0], p[1], p[2]));
            let end = UnitQuaternion::from_quaternion(nalgebra::Quaternion::new(n[3], n[0], n[1], n[2]));
            start.slerp(&end, t).to_homogeneous()
        },
        _ => Matrix4::identity(),
    }
}

fn interpolate_scale(prev: &AnimationTransform, next: &AnimationTransform, t: f32) -> Matrix4<f32> {
    match (prev, next) {
        (AnimationTransform::Scale(p), AnimationTransform::Scale(n)) => {
            let start = Vector3::from(*p);
            let end = Vector3::from(*n);
            let interpolated = start.lerp(&end, t);
            Matrix4::new_nonuniform_scaling(&interpolated)
        },
        _ => Matrix4::identity(),
    }
}