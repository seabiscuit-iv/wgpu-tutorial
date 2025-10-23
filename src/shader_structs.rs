use bytemuck::{Pod, Zeroable};
use wgpu::*;


#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
    tex_coords: [f32; 2]
}

// triangle
pub const VERTICES: &[Vertex] = &[
    // Front face (Z+)
    Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 1.0] },

    // Back face (Z-)
    Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 1.0] },

    // Left face (X-)
    Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 1.0] },

    // Right face (X+)
    Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 1.0] },

    // Top face (Y+)
    Vertex { position: [-0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [ 0.5,  0.5,  0.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [ 0.5,  0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [-0.5,  0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 1.0] },

    // Bottom face (Y-)
    Vertex { position: [-0.5, -0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [ 0.5, -0.5, -0.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [ 0.5, -0.5,  0.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [-0.5, -0.5,  0.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 1.0] },

    Vertex { position: [-2.5, -2.5, -2.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 0.0] },
    Vertex { position: [ 2.5, -2.5, -2.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 0.0] },
    Vertex { position: [ 2.5, -2.5,  2.5], color: [1.0, 1.0, 1.0], tex_coords: [1.0, 1.0] },
    Vertex { position: [-2.5, -2.5,  2.5], color: [1.0, 1.0, 1.0], tex_coords: [0.0, 1.0] },
];

pub const INDICES: &[u16] = &[
    // Front
    0, 1, 2,
    2, 3, 0,
    // Back
    4, 5, 6,
    6, 7, 4,
    // Left
    8, 9, 10,
    10, 11, 8,
    // Right
    12, 13, 14,
    14, 15, 12,
    // Top
    16, 17, 18,
    18, 19, 16,
    // Bottom
    20, 21, 22,
    22, 23, 20,

    24, 26, 25,
    26, 24, 27,
];


impl Vertex {
    const ATTRIBS : [VertexAttribute; 3] = vertex_attr_array![
        0 => Float32x3,
        1 => Float32x3,
        2 => Float32x2
    ];

    pub fn desc() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS
        }
    }
}


