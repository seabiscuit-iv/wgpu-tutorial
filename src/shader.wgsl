

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

// @builtin(position) is in framebuffer space aka pixel space
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>
};



@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.clip_position = vec4<f32>(model.position, 1.0);
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(convert_color(in.color), 1.0);
}

fn convert_color(srgb_color: vec3<f32>) -> vec3<f32> {
    var rgb_color: vec3<f32> = ((srgb_color + 0.055) / 1.055);
    rgb_color.r = pow(rgb_color.r, 2.4);
    rgb_color.g = pow(rgb_color.g, 2.4);
    rgb_color.b = pow(rgb_color.b, 2.4);
    return rgb_color;
}