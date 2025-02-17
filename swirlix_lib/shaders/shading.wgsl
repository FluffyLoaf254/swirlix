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
    let sample = textureSample(render_texture, render_sampler, input.uv);

    if (sample.a >= 1.0) {
        return vec4<f32>(0.3, 0.4, 0.5, 1.0);
    }

    return simple_blinn_phong(sample.rgb, compute_normal(input.uv));
}

fn simple_blinn_phong(color: vec3<f32>, normal: vec3<f32>) -> vec4<f32> {
    const specular_power = 2.0;
    const gloss = 0.95;

    let light_direction = normalize(vec3<f32>(0.8, 0.8, 1.0));
    let light_color = vec3<f32>(1.0, 1.0, 1.0);
    let n_dot_l = max(dot(normal, light_direction), 0.0);
    let view_direction = vec3<f32>(0.0, 0.0, 1.0); // set this based on camera
    let h = (light_direction - view_direction) / 2.;
    let specular = pow(dot(normal, h), specular_power) * gloss;

    return vec4<f32>(color * light_color * n_dot_l, 1.0) + specular;
}

fn compute_normal(uv: vec2<f32>) -> vec3<f32> {
    const delta = (4.0 / dimensions);

    let c0 = textureSample(render_texture, render_sampler, uv).w;
    let l1 = textureSample(render_texture, render_sampler, uv - vec2<f32>(delta, 0.0)).w;
    let l2 = textureSample(render_texture, render_sampler, uv - vec2<f32>(delta * 2.0, 0.0)).w;
    let r1 = textureSample(render_texture, render_sampler, uv + vec2<f32>(delta, 0.0)).w;
    let r2 = textureSample(render_texture, render_sampler, uv + vec2<f32>(delta * 2.0, 0.0)).w;
    let b1 = textureSample(render_texture, render_sampler, uv - vec2<f32>(0.0, delta)).w;
    let b2 = textureSample(render_texture, render_sampler, uv - vec2<f32>(0.0, delta * 2.0)).w;
    let t1 = textureSample(render_texture, render_sampler, uv + vec2<f32>(0.0, delta)).w;
    let t2 = textureSample(render_texture, render_sampler, uv + vec2<f32>(0.0, delta * 2.0)).w;
    
    let dl = abs((l1 * l2) / (2.0 * l2 - l1) - c0);
    let dr = abs((r1 * r2) / (2.0 * r2 - r1) - c0);
    let db = abs((b1 * b2) / (2.0 * b2 - b1) - c0);
    let dt = abs((t1 * t2) / (2.0 * t2 - t1) - c0);
    
    let center = get_world_pos(uv, c0);

    var dpdx = vec3<f32>();
    if (dl < dr) {
        dpdx = center - get_world_pos(uv - vec2<f32>(delta, 0.0), l1);
    } else {
        dpdx = -center + get_world_pos(uv + vec2<f32>(delta, 0.0), r1);
    }

    var dpdy = vec3<f32>();
    if (db < dt) {
        dpdy = center - get_world_pos(uv - vec2<f32>(0.0, delta), b1);
    } else {
        dpdy = -center + get_world_pos(uv + vec2<f32>(0.0, delta), t1);
    }

    return normalize(cross(dpdx, dpdy));
}

fn get_world_pos(uv: vec2<f32>, depth: f32) -> vec3<f32>
{
    return vec3<f32>(uv.x, uv.y, depth);
}
