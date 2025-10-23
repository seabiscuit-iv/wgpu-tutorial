use nalgebra::*;
use wgpu::{VertexAttribute, VertexBufferLayout, vertex_attr_array};

pub struct Instance  {
    pub position: Vector3<f32>,
    pub rotation: Quaternion<f32>
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4]
}


impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (Matrix4::new_translation(&self.position) * UnitQuaternion::from_quaternion(self.rotation).to_rotation_matrix().to_homogeneous()).into(), 
        }
    }
}


impl InstanceRaw {
    const ATTRIBS : [VertexAttribute; 4] = vertex_attr_array![
        5 => Float32x4,
        6 => Float32x4,
        7 => Float32x4,
        8 => Float32x4,
    ];


    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout { 
            array_stride: std::mem::size_of::<InstanceRaw>() as wgpu::BufferAddress, 
            step_mode: wgpu::VertexStepMode::Instance, 
            attributes: &Self::ATTRIBS
        }
    }
}