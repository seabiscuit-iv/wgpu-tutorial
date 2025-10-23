

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
    @location(2) tex_coords: vec2<f32>,
    @location(3) normal: vec3<f32>,
}

// @builtin(position) is in framebuffer space aka pixel space
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) pos: vec3<f32>
};

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,
}

struct CameraUniform {
    view_proj: mat4x4<f32>
}

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.color = model.color;
    out.normal = model.normal;
    out.pos = model.position;

    out.clip_position = camera.view_proj * model_matrix * vec4<f32>(model.position, 1.0);

    return out;
}


@group(0) @binding(0)
var diff_tex: texture_2d<f32>;

@group(0) @binding(1)
var diff_sampler: sampler;


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var light_pos = vec3<f32>(2.0, 2.0, 2.0);

    var tex_coords = in.tex_coords;
    tex_coords.y = 1.0 - tex_coords.y;

    let tex_color : vec4<f32> = textureSample(diff_tex, diff_sampler, tex_coords);

    let N = normalize(in.normal);
    let light_dir = normalize(light_pos - in.pos);

    let diff = max(dot(N, light_dir), 0.0);

    return vec4<f32>(tex_color.xyz * (0.1 + diff), 1.0);
}

fn convert_color(srgb_color: vec4<f32>) -> vec4<f32> {
    var rgb_color: vec4<f32> = ((srgb_color + 0.055) / 1.055);
    rgb_color.r = pow(rgb_color.r, 2.4);
    rgb_color.g = pow(rgb_color.g, 2.4);
    rgb_color.b = pow(rgb_color.b, 2.4);
    return rgb_color;
}