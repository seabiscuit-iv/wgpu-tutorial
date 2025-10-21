use std::f32::consts::PI;

use bytemuck::{Pod, Zeroable};
use wgpu::*;
use nalgebra::*;


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



fn spherical_to_cartesian(radius: f32, theta: f32, phi: f32) -> Point3<f32> {
    Point3::new(
        radius * phi.sin() * theta.cos(),
        radius * phi.cos(),
        radius * phi.sin() * theta.sin(),
    )
}


pub struct CameraController {
    pub w: bool,
    pub a: bool,
    pub s: bool,
    pub d: bool,
    pub e: bool,
    pub q: bool,
}


pub struct Camera {
    pub sphericals: Vector3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect_ratio: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
    pub cam_controller: CameraController
}


impl Camera {
    pub fn build_view_proj_matrix(&self) -> Matrix4<f32>{
        let eye = spherical_to_cartesian(self.sphericals.x, self.sphericals.y, self.sphericals.z);
        let view = Matrix4::look_at_rh(&eye, &self.target, &self.up);

        let persp = Perspective3::new(self.aspect_ratio, self.fovy, self.znear, self.zfar);
        let proj = persp.to_homogeneous();

        let opengl_to_wgpu = Matrix4::new(
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 0.5, 0.0,
            0.0, 0.0, 0.5, 1.0,
        );

        let view_proj = proj * view;

        opengl_to_wgpu * view_proj
    }

    pub fn update(&mut self) {
        if self.cam_controller.e {
            self.sphericals.x += 0.01;
        }
        if self.cam_controller.q {
            self.sphericals.x -= 0.01;
        }
        if self.cam_controller.d {
            self.sphericals.y += 0.01;
        }
        if self.cam_controller.a {
            self.sphericals.y -= 0.01;
        }
        if self.cam_controller.w {
            self.sphericals.z += 0.01
        }
        if self.cam_controller.s {
            self.sphericals.z -= 0.01
        }

        self.sphericals.x = self.sphericals.x.clamp(0.1, 50.0);
        self.sphericals.z = self.sphericals.z.clamp(PI/10.0, 9.0 * PI/10.0);
    }
}



#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4]
}

impl CameraUniform {
    pub fn new() -> Self {
        Self {
            view_proj: Matrix4::identity().into()
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_proj_matrix().into();
    }
}

