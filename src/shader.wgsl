

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

fn rng(seed: f32) -> f32 {
    let x = fract(sin(seed * 12.9898) * 43758.5453);
    return x;
}


@group(0) @binding(0)
var diff_tex: texture_2d<f32>;

@group(0) @binding(1)
var diff_sampler: sampler;

@group(2) @binding(0)
var<uniform> time: f32;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var light_pos = vec3<f32>(2 * cos(time), 3.0, 2 * sin(time)) * 4.0;

    var tex_coords = in.tex_coords;
    tex_coords.y = 1.0 - tex_coords.y;

    let tex_color : vec4<f32> = textureSample(diff_tex, diff_sampler, tex_coords);

    let N = normalize(in.normal);
    let light_dir = normalize(light_pos - in.pos);

    var diff = max(dot(N, light_dir), 0.0);

    const PROBE_DENSITY = 0.1;
    const STOCHASTIC_SAMPLE_RADIUS = 0.3;

    var offset = STOCHASTIC_SAMPLE_RADIUS * vec3<f32>(rng(time * in.pos.x), rng(time * in.pos.y), rng(time * in.pos.z)) - (STOCHASTIC_SAMPLE_RADIUS / 2.0);

    var shadowray_pos = in.pos + N * 0.001 + offset;

    shadowray_pos = round(shadowray_pos / PROBE_DENSITY) * PROBE_DENSITY;

    let shadowray_dir = normalize(light_pos - shadowray_pos);

    let ray = Ray(shadowray_pos, shadowray_dir);
    let hitinfo = intersect_unit_cube(ray);
    
    if hitinfo.hit && hitinfo.t_near > 0.001 {
        diff = 0.0;
    }

    return vec4<f32>(tex_color.xyz * (0.1 + diff), 1.0);
}

fn convert_color(srgb_color: vec4<f32>) -> vec4<f32> {
    var rgb_color: vec4<f32> = ((srgb_color + 0.055) / 1.055);
    rgb_color.r = pow(rgb_color.r, 2.4);
    rgb_color.g = pow(rgb_color.g, 2.4);
    rgb_color.b = pow(rgb_color.b, 2.4);
    return rgb_color;
}

struct Ray {
    origin: vec3<f32>,
    dir: vec3<f32>,
};

struct HitInfo {
    hit: bool,
    t_near: f32,
    t_far: f32,
};

fn intersect_unit_cube(ray: Ray) -> HitInfo {
    var tmin = -1e10;
    var tmax =  1e10;

    let bounds_min = vec3<f32>(-0.5, -0.5, -0.5);
    let bounds_max = vec3<f32>( 0.5,  0.5,  0.5);

    for (var i = 0; i < 3; i = i + 1) {
        if (abs(ray.dir[i]) < 1e-6) {
            if (ray.origin[i] < bounds_min[i] || ray.origin[i] > bounds_max[i]) {
                return HitInfo(false, 0.0, 0.0);
            }
        } else {
            let invD = 1.0 / ray.dir[i];
            var t0 = (bounds_min[i] - ray.origin[i]) * invD;
            var t1 = (bounds_max[i] - ray.origin[i]) * invD;

            if (t0 > t1) {
                let tmp = t0;
                t0 = t1;
                t1 = tmp;
            }

            tmin = max(tmin, t0);
            tmax = min(tmax, t1);

            if (tmax < tmin) {
                return HitInfo(false, 0.0, 0.0);
            }
        }
    }

    return HitInfo(true, tmin, tmax);
}
