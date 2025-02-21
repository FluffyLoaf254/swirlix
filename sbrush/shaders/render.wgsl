struct Settings {
    resolution: u32,
}

struct VertexInput {
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    let x = f32(i32(input.index & 1u) * 2 - 1);
    let y = f32(i32(input.index & 2u) - 1);
    let u = x / 2.0 + 0.5;
    let v = 1.0 - (y / 2.0 + 0.5);
    return VertexOutput(vec4<f32>(x, y, 0.0, 1.0), vec2<f32>(u, v));
}

@group(0) @binding(0) var<uniform> settings: Settings;
@group(0) @binding(1) var render_sampler: sampler;
@group(0) @binding(2) var render_texture: texture_2d<f32>;

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let delta = 1.0 / f32(settings.resolution);
    let distance = 1.0;

    var total = vec3<f32>(0.0, 0.0, 0.0);
    var count = 0.0;

    for (var x = -delta * distance; x <= delta * distance; x += delta) {
        for (var y = -delta * distance; x <= delta * distance; x += delta) {
            total += textureSample(render_texture, render_sampler, input.uv + vec2(x, y)).rgb;
            count += 1.0;
        }
    }

    return vec4<f32>(total / count, 1.0);
}
