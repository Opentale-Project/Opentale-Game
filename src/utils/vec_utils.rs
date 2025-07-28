use bevy::prelude::*;

pub fn vec_round_to_int(vec: &Vec3) -> IVec3 {
    let rounded = vec.round();
    IVec3::new(rounded.x as i32, rounded.y as i32, rounded.z as i32)
}
