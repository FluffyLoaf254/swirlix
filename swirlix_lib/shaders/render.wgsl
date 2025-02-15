struct VertexInput {
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) point: vec3<f32>,
}

@group(0) @binding(0) var<storage, read> voxels: array<u32>;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    let x = f32(i32(input.index & 1u) * 2 - 1);
    let y = f32(i32(input.index & 2u) - 1);
    return VertexOutput(vec4<f32>(x, y, 0.0, 1.0), vec3<f32>(x / 2.0 + 0.5, 1.0 - (y / 2.0 + 0.5), 0.5));
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var color = vec4<f32>(1.0, 1.0, 1.0, 1.0);
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

        var candidates = 255u;
        if (input.point.x - center.x <= 0.0) {
            // 1 lfb
            // 4 lbb
            // 16 lft
            // 64 lbt
            candidates &= 85u;
        } else {
            // 2 rfb
            // 8 rbb
            // 32 rft
            // 128 rbt
            candidates &= 170u;
        }
        if (input.point.y - center.y <= 0.0) {
            // 1 lfb
            // 2 rfb
            // 16 lft
            // 32 rft
            candidates &= 51u;
        } else {
            // 4 lbb
            // 8 rbb
            // 64 lbt
            // 128 rbt
            candidates &= 204u;
        }
        if (input.point.z - center.z <= 0.0) {
            // 1 lfb
            // 2 rfb
            // 4 lbb
            // 8 rbb
            candidates &= 15u;
        } else {
            // 16 lft
            // 32 rft
            // 64 lbt
            // 128 rbt
            candidates &= 240u;
        }

        let matches = (children & candidates);
        if (matches == 0u) {
            break;
        }

        if ((matches & 1u) != 0) { // lfb
            center.x -= quarter_size;
            center.y -= quarter_size;
            center.z -= quarter_size;
        } else if ((matches & 2u) != 0) { // rfb
            center.x += quarter_size;
            center.y -= quarter_size;
            center.z -= quarter_size;
        } else if ((matches & 4u) != 0) { // lbb
            center.x -= quarter_size;
            center.y += quarter_size;
            center.z -= quarter_size; 
        } else if ((matches & 8u) != 0) { // rbb
            center.x += quarter_size;
            center.y += quarter_size;
            center.z -= quarter_size;
        } else if ((matches & 16u) != 0) { // lft
            center.x -= quarter_size;
            center.y -= quarter_size;
            center.z += quarter_size;
        } else if ((matches & 32u) != 0) { // rft
            center.x += quarter_size;
            center.y -= quarter_size;
            center.z += quarter_size;
        } else if ((matches & 64u) != 0) { // lbt
            center.x -= quarter_size;
            center.y += quarter_size;
            center.z += quarter_size; 
        } else if ((matches & 128u) != 0) { // rbt
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
