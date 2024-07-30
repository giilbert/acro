// Very temporary until a resource system is implemented!!

struct VertexInput {
    @location(0) position: vec3<f32>,
};

@group(0) @binding(0)
var<uniform> model_matrix: mat4x4<f32>;

@group(1) @binding(0)
var<uniform> view_matrix: mat4x4<f32>;
@group(1) @binding(1)
var<uniform> projection_matrix: mat4x4<f32>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = projection_matrix * view_matrix * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}


@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(0.8, 0.1, 0.1, 1.0);
}
