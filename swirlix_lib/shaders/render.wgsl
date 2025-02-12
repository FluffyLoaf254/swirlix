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
    var color = vec4<f32>(input.uv.x, input.uv.y, 1.0, 1.0);
    var center = vec3<f32>(0.5, 0.5, 0.5);
    var size = 0.5;
    var pointer = 0u;

    var found = false;

    while (!found) {
        let current = voxels[pointer];
        pointer = (current >> 16);
        let children = ((current >> 8) & 255u);
        let leaves = (current & 255u);
        var candidates = 0u;
        if (input.uv.x < center.x && input.uv.y < center.y) {
            candidates |= 5u; // lfb, lft
        } else if (input.uv.x > center.x && input.uv.y < center.y) {
            candidates |= 34u; // rfb, rft
        } else if (input.uv.x < center.x && input.uv.y > center.y) {
            candidates |= 68u; // lbb, lbt
        } else if (input.uv.x > center.x && input.uv.y > center.y) {
            candidates |= 136u; // rbb, rbt 
        }

        let matches = children & candidates;
        var child = 0u;
        if (matches == 0u) {
            found = true;
        } else {
            size /= 2.0;
            if ((matches & 170u) != 0 && (matches & 204u) != 0) {
                center.x += size;
                center.y += size;
                center.z += size;
                child = 128u;
            } else if ((matches & 170u) != 0 && (matches & 204u) == 0) {
                center.x += size;
                center.y -= size;
                center.z += size;
                child = 32u;
            } else if ((matches & 170u) == 0 && (matches & 204u) != 0) {
                center.x -= size;
                center.y += size;
                center.z += size;
                child = 64u;
            } else if ((matches & 170u) == 0 && (matches & 204u) != 0) {
                center.x -= size;
                center.y -= size;
                center.z += size;
                child = 16u;
            }

            var current = 1u;
            var children_left = children;
            var child_count = 0u;
            while (current != child) {
                child_count += (children_left & 1u);
                children_left = (children_left >> 1);
                current = (current << 1);
            }

            pointer += child_count;

            if ((child & leaves) != 0u) {
                color = vec4<f32>(1.0, 0.0, 0.0, 1.0);
                found = true;
            }
        }
    }

    return color;
}
