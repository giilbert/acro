struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

// Context bind group
@group(0) @binding(0)
var<uniform> screen_size: vec2<f32>;

struct InstanceInput {
    @location(1) offset: vec2<f32>,
    @location(2) size: vec2<f32>,
    // @location(3) color: vec4<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    var scale = instance.size / screen_size * 2;
    var translation = (instance.offset - screen_size / 2) / screen_size * 2;

    out.clip_position = vec4<f32>(
        model.position * scale + vec2<f32>(translation.x, -translation.y),
        0.0,
        1.0
    );
    out.color = vec4<f32>(1.0, 0.0, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // return vec4<f32>(1.0, 1.0, 1.0, 0.4);
    return in.color;
}
