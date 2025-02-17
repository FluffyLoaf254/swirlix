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

@fragment fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let initial = textureSample(render_texture, render_sampler, input.uv);

    const delta = 32;
    const allowed = 0.015;

    let p = vec2<i32>(i32(input.uv.x * dimensions), i32(input.uv.y * dimensions));

    var total = 0.0;
    var count = 0.0;

    for (var x = 0; x <= delta; x++) {
        for (var y = 0; y <= delta; y++) {
            let pos_depth = textureSample(render_texture, render_sampler, get_uv(p + vec2<i32>(x, y))).w;
            if (abs(initial.w - pos_depth) <= allowed) {
                count += 1.0;
                total += pos_depth;
            }
            let neg_depth = textureSample(render_texture, render_sampler, get_uv(p - vec2<i32>(x, y))).w;
            if (abs(initial.w - neg_depth) <= allowed) {
                count += 1.0;
                total += neg_depth;
            }
        }
    }

    return vec4<f32>(initial.rgb, total / count);
}

fn get_uv(pixel: vec2<i32>) -> vec2<f32> {
    return vec2<f32>(f32(pixel.x) / dimensions, f32(pixel.y) / dimensions);
}
