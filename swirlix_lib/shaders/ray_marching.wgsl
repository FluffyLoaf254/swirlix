struct VertexInput {
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct VoxelHit {
    hit: bool,
    pointer: u32,
    distance: f32,
    center: vec3<f32>,
    size: f32,
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
    
    const max_steps = 32u;
    const minimum_distance = 0.0;
    const maximum_distance = 1.0;
    const smallest_voxel_size = 0.001;

    var ray_distance = 0.0;

    for (var step = 0u; step < max_steps; step += 1u) {
        let position = ray_origin + ray_distance * ray_direction;

        let closest = hit_voxel(position);

        if (!closest.hit) {
            break;
        }

        if (closest.distance <= minimum_distance) {
            return vec4<f32>(1.0, 0.0, 0.0, voxel_distance(ray_origin, closest.center, closest.size / 2.0));
        }

        ray_distance += max(closest.distance, smallest_voxel_size);

        if (ray_distance > maximum_distance) {
            break;
        }
    }

    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

fn hit_voxel(position: vec3<f32>) -> VoxelHit {
    var pointer = 0u;
    var voxel_size = 1.0;

    let voxel_center = vec3<f32>(0.5, 0.5, 0.5);

    const max_steps = 12u;

    var minimum_distance = 100.0;
    var level = 0u;

    // parent data
    var visited_pointers = array<u32, 12>();
    var visited_children = array<u32, 12>();
    var visited_centers = array<vec3<f32>, 12>();

    visited_pointers[level] = pointer;
    visited_children[level] = 0u;
    visited_centers[level] = voxel_center;

    var result = VoxelHit(false, pointer, 0.0, voxel_center, voxel_size);

    for (var step = 0u; step < max_steps; step += 1u) {
        var parent_pointer = visited_pointers[level];
        var parent = voxels[parent_pointer];
        pointer = voxels[parent_pointer + 1u];
        var siblings = ((parent >> 8) & 255u);

        // visited all siblings, go up a level
        while ((level > 0u) && (siblings == visited_children[level])) {
            level -= 1u;
            parent_pointer = visited_pointers[level];
            parent = voxels[parent_pointer];
            pointer = voxels[parent_pointer + 1u];
            siblings = ((parent >> 8) & 255u);
            voxel_size *= 2.0;
        }

        let half_voxel_size = voxel_size / 2.0;
        let quarter_voxel_size = voxel_size / 4.0;

        var chosen_sibling = 0u;
        var sibling_offset = 0u;
        var sibling_center = visited_centers[level];
        
        for (var sibling = 0u; sibling < 8u; sibling += 1u) {
            let sibling_value = (1u << sibling);
            if ((siblings & sibling_value) == 0u) {
                continue;
            }

            // haven't visited this sibling
            if ((visited_children[level] & sibling_value) == 0u) {
                chosen_sibling = sibling_value;

                if ((sibling_value & 85u) != 0u) { // left
                    sibling_center.x -= quarter_voxel_size;
                } else { // right
                    sibling_center.x += quarter_voxel_size;
                }
                if ((sibling_value & 51u) != 0u) { // back
                    sibling_center.y -= quarter_voxel_size;
                } else { // front
                    sibling_center.y += quarter_voxel_size;
                }
                if ((sibling_value & 15u) != 0u) { // top
                    sibling_center.z -= quarter_voxel_size;
                } else { // bottom
                    sibling_center.z += quarter_voxel_size;
                }
                break;
            }

            sibling_offset += 1u;
        }

        visited_children[level] = (visited_children[level] | chosen_sibling);

        let sibling_pointer = (pointer + sibling_offset * 2u);

        let hit = hit_next_voxel(sibling_pointer, sibling_center, half_voxel_size, position);

        if (hit.hit) { // is a leaf
            if (hit.distance <= minimum_distance) {
                result = hit;
                minimum_distance = hit.distance;
            }
        } else { // not a leaf, go down a level
            level += 1u;
            visited_pointers[level] = sibling_pointer;
            visited_children[level] = 0u;
            visited_centers[level] = sibling_center;
            voxel_size = half_voxel_size;
        }
    }

    return result;
}

fn hit_next_voxel(pointer: u32, voxel_center: vec3<f32>, voxel_size: f32, position: vec3<f32>) -> VoxelHit {
    var current = voxels[pointer];
    var next_pointer = voxels[pointer + 1u];

    let half_voxel_size = voxel_size / 2.0;
    let quarter_voxel_size = voxel_size / 4.0;

    let children = ((current >> 8) & 255u);
    let leaves = (current & 255u);

    var minimum_distance = 100.0;
    var child_offset = 0u;

    var hit = VoxelHit(false, current, 0.0, voxel_center, voxel_size);

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

        if (child_distance <= minimum_distance) {
            minimum_distance = child_distance;
            hit = VoxelHit((leaves & child_value) != 0u, next_pointer + child_offset * 2u, child_distance, child_center, half_voxel_size);
        }

        child_offset += 1u;
    }

    return hit;
}

fn voxel_distance(point: vec3<f32>, center: vec3<f32>, half_size: f32) -> f32 {
    let x = max(point.x - center.x - half_size, center.x - point.x - half_size);
    let y = max(point.y - center.y - half_size, center.y - point.y - half_size);
    let z = max(point.z - center.z - half_size, center.z - point.z - half_size);

    var distance = x;
    distance = max(distance, y);
    distance = max(distance, z);

    return max(0.0, distance);
}
