struct VertexInput {
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct VoxelHit {
    voxel: u32,
    distance: f32,
    center: vec3<f32>,
    half_size: f32,
}

@group(0) @binding(0) var<storage, read> voxels: array<u32>;

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    let x = f32(i32(input.index & 1u) * 2 - 1);
    let y = f32(i32(input.index & 2u) - 1);
    let u = x / 2.0 + 0.5;
    let v = 1.0 - (y / 2.0 + 0.5);
    return VertexOutput(vec4<f32>(x, y, 0.0, 1.0), vec2<f32>(u, v));
}

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let ray_origin = vec3<f32>(input.uv, 0.0); // multiply by world matrix
    let ray_direction = vec3<f32>(0.0, 0.0, 1.0); // multiply by world matrix
    
    const max_steps = 50u;
    const minimum_distance = 0.0;
    const maximum_distance = 1.0;
    const smallest_voxel_size = 0.02;

    var ray_distance = 0.0;

    if (voxels[0u] == 0u) {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    for (var step = 0u; step < max_steps; step += 1u) {
        let position = ray_origin + ray_distance * ray_direction;

        let closest = hit_voxel(position);

        if (closest.distance <= minimum_distance) {
            return vec4<f32>(1.0, 0.0, 0.0, voxel_distance(ray_origin, closest.center, closest.half_size));
        }

        ray_distance += max(closest.distance * (closest.half_size / 2.0), smallest_voxel_size);

        if (ray_distance > maximum_distance) {
            break;
        }
    }

    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

fn hit_voxel(position: vec3<f32>) -> VoxelHit {
    var pointer = 0u;
    var current = voxels[pointer];
    var voxel_center = vec3<f32>(0.5, 0.5, 0.5);
    var voxel_size = 1.0;

    const max_steps = 12u;

    for (var step = 0u; step < max_steps; step += 1u) {
        let half_voxel_size = voxel_size / 2.0;
        let quarter_voxel_size = voxel_size / 4.0;

        pointer = (current >> 16);
        let children = ((current >> 8) & 255u);
        let leaves = (current & 255u);

        var minimum_distance = 1.0;
        var child_offset = 0u;
        var pointer_offset = 0u;
        var new_center = voxel_center;
        var chosen_child = 0u;

        for (var child = 0u; child < 8u; child += 1u) {
            let child_value = (1u << child);
            if ((children & child_value) == 0u) {
                continue;
            }

            var child_center = voxel_center;
            if ((child_value & 85u) != 0u) { // left
                child_center.x -= quarter_voxel_size;
            } else { // right
                child_center.x += quarter_voxel_size;
            }
            if ((child_value & 51u) != 0u) { // back
                child_center.y -= quarter_voxel_size;
            } else { // front
                child_center.y += quarter_voxel_size;
            }
            if ((child_value & 15u) != 0u) { // top
                child_center.z -= quarter_voxel_size;
            } else { // bottom
                child_center.z += quarter_voxel_size;
            }
            let child_distance = voxel_distance(position, child_center, quarter_voxel_size);
            if (child_distance < minimum_distance) {
                chosen_child = child_value;
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

        if ((leaves & chosen_child) != 0u) {
            break;
        }
    }

    let half_size = voxel_size / 2.0;

    return VoxelHit(current, voxel_distance(position, voxel_center, half_size), voxel_center, half_size);
}

fn voxel_distance(point: vec3<f32>, center: vec3<f32>, half_size: f32) -> f32 {
    let x = max(point.x - center.x - half_size, center.x - point.x - half_size);
    let y = max(point.y - center.y - half_size, center.y - point.y - half_size);
    let z = max(point.z - center.z - half_size, center.z - point.z - half_size);

    var distance = x;
    distance = max(distance, y);
    distance = max(distance, z);

    return distance;
}
