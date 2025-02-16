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

@fragment fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(render_texture, render_sampler, input.uv);

    if (sample.a >= 1.0) {
        return vec4<f32>(0.5, 0.6, 0.75, 1.0);
    }

    return simple_lambert(sample.rgb, compute_normal(input.uv));
    // return vec4<f32>(compute_normal(input.uv), 1.0);
}

fn compute_normal_v2(uv: vec2<f32>) -> vec3<f32> {
    let p = vec2<i32>(i32(uv.x * 1000.0), i32(uv.y * 1000.0));
    let fx0 = textureSample(render_texture, render_sampler, get_uv(p - vec2<i32>(1, 0))).w;
    let fx1 = textureSample(render_texture, render_sampler, get_uv(p + vec2<i32>(1, 0))).w;
    let fy0 = textureSample(render_texture, render_sampler, get_uv(p - vec2<i32>(0, 1))).w;
    let fy1 = textureSample(render_texture, render_sampler, get_uv(p + vec2<i32>(0, 1))).w;
    let epsilon = 0.001;

    return normalize(vec3<f32>((fx0 - fx1) / (2.0 * epsilon), (fy0 - fy1) / (2.0 * epsilon), 1.0));
}

fn compute_normal(uv: vec2<f32>) -> vec3<f32> {
    let p = vec2<i32>(i32(uv.x * 1000.0), i32(uv.y * 1000.0));
    let c0 = textureSample(render_texture, render_sampler, get_uv(p)).w;
    let l2 = textureSample(render_texture, render_sampler, get_uv(p - vec2<i32>(2, 0))).w;
    let l1 = textureSample(render_texture, render_sampler, get_uv(p - vec2<i32>(1, 0))).w;
    let r1 = textureSample(render_texture, render_sampler, get_uv(p + vec2<i32>(1, 0))).w;
    let r2 = textureSample(render_texture, render_sampler, get_uv(p + vec2<i32>(2, 0))).w;
    let b2 = textureSample(render_texture, render_sampler, get_uv(p - vec2<i32>(0, 2))).w;
    let b1 = textureSample(render_texture, render_sampler, get_uv(p - vec2<i32>(0, 1))).w;
    let t1 = textureSample(render_texture, render_sampler, get_uv(p + vec2<i32>(0, 1))).w;
    let t2 = textureSample(render_texture, render_sampler, get_uv(p + vec2<i32>(0, 2))).w;
    
    let dl = abs(l1 * l2 / (2.0 * l2 - l1) - c0);
    let dr = abs(r1 * r2 / (2.0 * r2 - r1) - c0);
    let db = abs(b1 * b2 / (2.0 * b2 - b1) - c0);
    let dt = abs(t1 * t2 / (2.0 * t2 - t1) - c0);
    
    let ce = get_world_pos(p, c0);

    var dpdx = vec3<f32>(0.0, 0.0, 0.0);
    if (dl < dr) {
        dpdx = ce - get_world_pos(p - vec2<i32>(1, 0), l1);
    } else {
        dpdx = -ce + get_world_pos(p + vec2<i32>(1, 0), r1);
    }

    var dpdy = vec3<f32>(0.0, 0.0, 0.0);
    if (db < dt) {
        dpdy = ce - get_world_pos(p - vec2<i32>(0, 1), b1);
    } else {
        dpdy = -ce + get_world_pos(p + vec2<i32>(0, 1), t1);
    }

    return normalize(cross(dpdx, dpdy));
}

fn get_world_pos(pixel: vec2<i32>, depth: f32) -> vec3<f32>
{
    return vec3<f32>(f32(pixel.x) / 1000.0, f32(pixel.y) / 1000.0, depth);
}

fn get_uv(pixel: vec2<i32>) -> vec2<f32> {
    return vec2<f32>(f32(pixel.x) / 1000.0, f32(pixel.y) / 1000.0);
}

fn simple_lambert(color: vec3<f32>, normal: vec3<f32>) -> vec4<f32> {
    const specular_power = 1.0;
    const gloss = 1.0;

    let light_position = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let light_color = vec3<f32>(1.0, 1.0, 1.0);
    let n_dot_l = max(dot(normal, light_position), 0.0);
    let view_direction = vec3<f32>(0.0, 0.0, 1.0); // set this based on camera
    let h = (light_position - view_direction) / 2.;
    let specular = pow(dot(normal, h), specular_power) * gloss;

    return vec4<f32>(color * light_color * n_dot_l * 0.95 + color * 0.05, 1.0) + specular;
}
