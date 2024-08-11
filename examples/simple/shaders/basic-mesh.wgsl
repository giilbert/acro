// Very temporary until a resource system is implemented!!

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> model_matrix: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> view_matrix: mat4x4<f32>;
@group(1) @binding(1)
var<uniform> projection_matrix: mat4x4<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) normal: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = projection_matrix * view_matrix * model_matrix * vec4<f32>(model.position, 1.0);

    out.normal = model.normal;

    return out;
}

@group(2) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(2) @binding(1)
var s_diffuse: sampler;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var sun_dir = vec3<f32>(0.0, 1.0, 0.0);

    var normal = normalize(in.normal);
    var light = normalize(sun_dir);
    var intensity = max(dot(normal, light), 0.0);

    return vec4<f32>(intensity, intensity, intensity, 1.0) * textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
