struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

// Context bind group
@group(0) @binding(0)
var<uniform> screen_size: vec2<f32>;

struct InstanceInput {
    @location(1) size: vec2<f32>,
    @location(2) offset: vec2<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    var size: vec2<f32> = vec2<f32>(instance.size.x / screen_size.x, instance.size.y / screen_size.y);
    out.clip_position = vec4<f32>(
        model.position * 0.5,
        0.0,
        1.0
    );
    // out.clip_position = projection_matrix * view_matrix * model_matrix * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
