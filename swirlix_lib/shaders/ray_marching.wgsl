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
    return VertexOutput(vec4<f32>(x, y, 0.0, 1.0), vec3<f32>(x / 2.0 + 0.5, 1.0 - (y / 2.0 + 0.5), 0.0));
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    var ray_origin = input.point; // multiply by world matrix
    var ray_direction = vec3<f32>(0.0, 0.0, 1.0); // multiply by world matrix
    var ray_distance = 0.0;

    const max_steps = 32u;
    const minimum_distance = 0.0025;
    const maximum_distance = 1.0;

    for (var step = 0u; step < max_steps; step += 1u) {
        let position = ray_origin + ray_distance * ray_direction;

        let distance_to_closest = distance_from_voxels(position);

        if (distance_to_closest < minimum_distance) {
            return vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }

        ray_distance += distance_to_closest;

        if (ray_distance > maximum_distance) {
            break;
        }
    }

    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

fn distance_from_voxels(position: vec3<f32>) -> f32 {
    var pointer = 0u;
    var current = voxels[pointer];
    var voxel_center = vec3<f32>(0.5, 0.5, 0.5);
    var voxel_size = 1.0;

    const max_steps = 10u;

    if (current == 0u) {
        return 1.0;
    }

    for (var step = 0u; step < max_steps; step += 1u) {
        let half_voxel_size = voxel_size / 2.0;
        let quarter_voxel_size = voxel_size / 4.0;

        if (current == 0u) {
            return voxel_distance(position, voxel_center, half_voxel_size);
        }

        pointer = (current >> 16);
        let children = ((current >> 8) & 255u);
        let leaves = (current & 255u);

        var minimum_distance = 1.0;
        var child_offset = 0u;
        var pointer_offset = 0u;
        var new_center = voxel_center;

        for (var child = 0u; child < 8u; child += 1u) {
            let child_value = (1u << child);
            if ((children & child_value) == 0u) {
                continue;
            }

            var child_center = voxel_center;
            if ((child_value & 170u) != 0u) { // right
                child_center.x += quarter_voxel_size;
            } else { // left
                child_center.x -= quarter_voxel_size;
            }
            if ((child_value & 204u) != 0u) { // front
                child_center.y += quarter_voxel_size;
            } else { // back
                child_center.y -= quarter_voxel_size;
            }
            if ((child_value & 240u) != 0u) { // bottom
                child_center.z += quarter_voxel_size;
            } else { // top
                child_center.z -= quarter_voxel_size;
            }
            let child_distance = voxel_distance(position, child_center, quarter_voxel_size);
            if (child_distance < minimum_distance) {
                minimum_distance = child_distance;
                pointer_offset = child_offset;
                new_center = child_center;
            }

            child_offset += 1u;
        }

        pointer += pointer_offset;
        voxel_size = half_voxel_size;
        voxel_center = new_center;

        current = voxels[pointer];
    }

    return 1.0;
}

fn voxel_distance(point: vec3<f32>, center: vec3<f32>, half_size: f32) -> f32 {
    let shifted = abs((point - center) / half_size);

    return sqrt(pow(max(0.0, shifted.x - 1.0), 2.0) + pow(max(0.0, shifted.y - 1.0), 2.0) + pow(max(0.0, shifted.z - 1.0), 2.0)) * half_size;
}
