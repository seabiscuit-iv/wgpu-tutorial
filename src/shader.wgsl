

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
}

// @builtin(position) is in framebuffer space aka pixel space
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) tex_coords: vec2<f32>
};


struct CameraUniform {
    view_proj: mat4x4<f32>
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = model.color;

    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);

    return out;
}


@group(0) @binding(0)
var diff_tex: texture_2d<f32>;

@group(0) @binding(1)
var diff_sampler: sampler;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var tex_coords = in.tex_coords;
    tex_coords.y = 1.0 - tex_coords.y;

    let tex_color = textureSample(diff_tex, diff_sampler, tex_coords);

    return tex_color;
}

fn convert_color(srgb_color: vec4<f32>) -> vec4<f32> {
    var rgb_color: vec4<f32> = ((srgb_color + 0.055) / 1.055);
    rgb_color.r = pow(rgb_color.r, 2.4);
    rgb_color.g = pow(rgb_color.g, 2.4);
    rgb_color.b = pow(rgb_color.b, 2.4);
    return rgb_color;
}