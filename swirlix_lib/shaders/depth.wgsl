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
    let sample = textureSample(render_texture, render_sampler, input.uv);

    const allowed = (5.0 / dimensions);
    const epsilon = 2;

    let p = vec2<i32>(i32(input.uv.x * dimensions), i32(input.uv.y * dimensions));
    let fx0 = textureSample(render_texture, render_sampler, get_uv(p - vec2<i32>(epsilon, 0))).w;
    let fx1 = textureSample(render_texture, render_sampler, get_uv(p + vec2<i32>(epsilon, 0))).w;
    let fy0 = textureSample(render_texture, render_sampler, get_uv(p - vec2<i32>(0, epsilon))).w;
    let fy1 = textureSample(render_texture, render_sampler, get_uv(p + vec2<i32>(0, epsilon))).w;

    var minimum = sample.w;
    var maximum = sample.w;
    if (abs(fx0 - sample.w) < allowed) {
        minimum = min(minimum, fx0);
        maximum = max(maximum, fx0);
    }
    if (abs(fx1 - sample.w) < allowed) {
        minimum = min(minimum, fx1);
        maximum = max(maximum, fx1);
    }
    if (abs(fy0 - sample.w) < allowed) {
        minimum = min(minimum, fy0);
        maximum = max(maximum, fy0);
    }
    if (abs(fy1 - sample.w) < allowed) {
        minimum = min(minimum, fy1);
        maximum = max(maximum, fy1);
    }

    return vec4<f32>(sample.rgb, (maximum + minimum) / 2.0);
}

fn get_uv(pixel: vec2<i32>) -> vec2<f32> {
    return vec2<f32>(f32(pixel.x) / dimensions, f32(pixel.y) / dimensions);
}
