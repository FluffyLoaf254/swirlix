struct VertexInput {
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    let x = f32(i32(input.index) - 1);
    let y = f32(i32(input.index & 1u) * 2 - 1);
    return VertexOutput(vec4<f32>(x, y, 0.0, 1.0), vec4<f32>(x / 2.0 + 0.5, y / 2.0 + 0.5, 0.0, 1.0));
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}
