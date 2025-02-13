struct VertexInput {
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0) var<storage, read> voxels: array<u32>;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    let x = f32(i32(input.index & 1u) * 2 - 1);
    let y = f32(i32(input.index & 2u) - 1);
    return VertexOutput(vec4<f32>(x, y, 0.0, 1.0), vec2<f32>(x / 2.0 + 0.5, 1.0 - (y / 2.0 + 0.5)));
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let z = 0.5;
    var color = vec4<f32>(input.uv.x, input.uv.y, 1.0, 1.0);
    var center = vec3<f32>(0.5, 0.5, 0.5);
    var size = 1.0;
    var pointer = 0u;
    var current = voxels[pointer];
    var iteration = 0u;

    if (current == 0u) {
        return color;
    }

    loop {
        if (current == 0u || iteration > 12u) {
            color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
            break;
        }
        let half_size = size / 2.0;
        let quarter_size = size / 4.0;

        pointer = (current >> 16);
        let children = ((current >> 8) & 255u);
        let leaves = (current & 255u);

        var candidates = 0u;
        if ((input.uv.x - center.x <= 0.0) && (input.uv.y - center.y <= 0.0) && (z - center.z <= 0.0)) {
            candidates |= 1u; // lfb
        } else if ((input.uv.x - center.x >= 0.0) && (input.uv.y - center.y <= 0.0) && (z - center.z <= 0.0)) {
            candidates |= 2u; // rfb
        } else if ((input.uv.x - center.x <= 0.0) && (input.uv.y - center.y >= 0.0) && (z - center.z <= 0.0)) {
            candidates |= 4u; // lbb
        } else if ((input.uv.x - center.x >= 0.0) && (input.uv.y - center.y >= 0.0) && (z - center.z <= 0.0)) {
            candidates |= 8u; // rbb
        } else if ((input.uv.x - center.x <= 0.0) && (input.uv.y - center.y <= 0.0) && (z - center.z >= 0.0)) {
            candidates |= 16u; // lft
        } else if ((input.uv.x - center.x >= 0.0) && (input.uv.y - center.y <= 0.0) && (z - center.z >= 0.0)) {
            candidates |= 32u; // rft
        } else if ((input.uv.x - center.x <= 0.0) && (input.uv.y - center.y >= 0.0) && (z - center.z >= 0.0)) {
            candidates |= 64u; // lbt
        } else if ((input.uv.x - center.x >= 0.0) && (input.uv.y - center.y >= 0.0) && (z - center.z >= 0.0)) {
            candidates |= 128u; // rbt 
        }

        let matches = (children & candidates);
        if (matches == 0u) {
            break;
        }

        if ((matches & 1u) != 0) {
            center.x -= quarter_size;
            center.y -= quarter_size;
            center.z -= quarter_size;
        } else if ((matches & 2u) != 0) {
            center.x += quarter_size;
            center.y -= quarter_size;
            center.z -= quarter_size;
        } else if ((matches & 4u) != 0) {
            center.x -= quarter_size;
            center.y += quarter_size;
            center.z -= quarter_size; 
        } else if ((matches & 8u) != 0) {
            center.x += quarter_size;
            center.y += quarter_size;
            center.z -= quarter_size;
        } else if ((matches & 16u) != 0) {
            center.x -= quarter_size;
            center.y -= quarter_size;
            center.z += quarter_size;
        } else if ((matches & 32u) != 0) {
            center.x += quarter_size;
            center.y -= quarter_size;
            center.z += quarter_size;
        } else if ((matches & 64u) != 0) {
            center.x -= quarter_size;
            center.y += quarter_size;
            center.z += quarter_size; 
        } else if ((matches & 128u) != 0) {
            center.x += quarter_size;
            center.y += quarter_size;
            center.z += quarter_size;
        }

        var child_offset = 0u;
        var looking_for_byte = 1u;
        var bits_left = children;
        while ((looking_for_byte & matches) == 0) {
            child_offset += (bits_left & 1u);
            bits_left = (bits_left >> 1);
            looking_for_byte = (looking_for_byte << 1);
        }

        pointer += child_offset;
        size = half_size;

        current = voxels[pointer];
        iteration++;
    }

    return color;
}
