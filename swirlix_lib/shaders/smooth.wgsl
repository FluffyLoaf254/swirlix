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

@group(0) @binding(0) var render_sampler: sampler;
@group(0) @binding(1) var render_texture: texture_2d<f32>;

const dimensions = 256.0;

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let initial = textureSample(render_texture, render_sampler, input.uv);

    const delta = 4.0;

    var maximum = initial.w;
    var minimum = initial.w;

    for (var x = -delta; x <= delta; x += 1.0) {
        for (var y = -delta; y <= delta; y += 1.0) {
            let allowed = (max(abs(x) + 1.0, abs(y) + 1.0) / dimensions);
            let uv = vec2<f32>(input.uv.x + (x / dimensions), input.uv.y + (y / dimensions));

            let depth = textureSample(render_texture, render_sampler, uv).w;
            if (abs(initial.w - depth) <= allowed) {
                maximum = max(maximum, depth);
                minimum = min(minimum, depth);
            }
        }
    }

    return vec4<f32>(initial.rgb, (maximum + minimum) / 2.0);
}
