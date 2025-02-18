struct Settings {
    resolution: u32,
}

struct VertexInput {
    @builtin(vertex_index) index: u32,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

struct Material {
    color: vec4<f32>,
    roughness: f32,
    metallic: f32,
}

struct VoxelHit {
    hit: bool,
    pointer: u32,
    distance: f32,
    center: vec3<f32>,
    size: f32,
    visited: u32,
    child_value: u32,
    color: u32,
    normal: vec3<f32>,
}

@vertex
fn vertex_main(input: VertexInput) -> VertexOutput {
    let x = f32(i32(input.index & 1u) * 2 - 1);
    let y = f32(i32(input.index & 2u) - 1);
    let u = x / 2.0 + 0.5;
    let v = 1.0 - (y / 2.0 + 0.5);
    return VertexOutput(vec4<f32>(x, y, 0.0, 1.0), vec2<f32>(u, v));
}

@group(0) @binding(0) var<uniform> settings: Settings;
@group(0) @binding(1) var<storage, read> voxels: array<u32>;
@group(0) @binding(2) var<storage, read> materials: array<Material>;

const hit_distance = 0.0;

@fragment
fn fragment_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let ray_origin = vec3<f32>(input.uv, 0.0); // multiply by world matrix
    let ray_direction = vec3<f32>(0.0, 0.0, 1.0); // multiply by world matrix
    
    const max_steps = 32u;
    const maximum_distance = 1.0;

    var ray_distance = 0.0;

    for (var step = 0u; step < max_steps; step += 1u) {
        var position = ray_origin + ray_distance * ray_direction;

        let closest = hit_root(ray_direction, position);

        if (!closest.hit) {
            break;
        }

        if (closest.distance <= hit_distance) {
            return simple_blinn_phong(materials[closest.color].color, normalize(closest.normal));
        }

        ray_distance += max(closest.distance, 1.0 / f32(settings.resolution));

        if (ray_distance > maximum_distance) {
            break;
        }
    }

    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}

fn hit_root(ray_direction: vec3<f32>, position: vec3<f32>) -> VoxelHit {
    let root = VoxelHit(false, 0u, 100.0, vec3<f32>(0.5, 0.5, 0.5), 1.0, 0u, 0u, 0u, vec3<f32>(0.0, 0.0, 0.0));

    var hit = hit_voxel(root, position);

    return hit;
}

fn hit_voxel(parent: VoxelHit, position: vec3<f32>) -> VoxelHit {
    const max_steps = 64u;

    var minimum_distance = 100.0;
    var level = 0u;

    var result = parent;
    var next = parent;

    var visited = array<VoxelHit, 64>();

    visited[level] = next;

    for (var step = 0u; step < max_steps; step += 1u) {
        var siblings = ((voxels[next.pointer] >> 8) & 255u);

        // visited all siblings, go up a level
        while ((level > 0u) && (siblings == next.visited)) {
            level -= 1u;
            next = visited[level];
        }

        let hit = hit_next_voxel(next, position);

        next.visited = (next.visited | hit.child_value);

        if (hit.hit) { // is a leaf
            if (hit.distance < minimum_distance) {
                result = hit;
                minimum_distance = hit.distance;
                if (hit.distance <= hit_distance) {
                    break;
                }
            }
        } else { // not a leaf, go down a level
            level += 1u;
            visited[level] = next;
            next = hit;
        }
    }

    return result;
}

fn hit_next_voxel(parent: VoxelHit, position: vec3<f32>) -> VoxelHit {
    var current = voxels[parent.pointer];
    var next_pointer = voxels[parent.pointer + 1u];

    let half_voxel_size = parent.size / 2.0;
    let quarter_voxel_size = parent.size / 4.0;

    let children = ((current >> 8) & 255u);
    let leaves = (current & 255u);

    var minimum_distance = 100.0;

    var hit = parent;
    var child_offset = 0u;
    var child_mask = 0u;
    var voxel_normal = vec3<f32>(0.0, 0.0, 0.0);
    var first_normal = true;

    for (var child = 0u; child < 8u; child += 1u) {
        let child_value = (1u << child);

        var child_normal = vec3<f32>(0.0, 0.0, 0.0);
        var child_center = parent.center;
        if ((child_value & 85u) != 0u) { // left
            child_center.x -= quarter_voxel_size;
            child_normal.x += 1.0;
        } else { // right
            child_center.x += quarter_voxel_size;
            child_normal.x -= 1.0;
        }
        if ((child_value & 51u) != 0u) { // back
            child_center.y -= quarter_voxel_size;
            child_normal.y += 1.0;
        } else { // front
            child_center.y += quarter_voxel_size;
            child_normal.y -= 1.0;
        }
        if ((child_value & 15u) != 0u) { // top
            child_center.z -= quarter_voxel_size;
            child_normal.z += 1.0;
        } else { // bottom
            child_center.z += quarter_voxel_size;
            child_normal.z -= 1.0;
        }

        if ((children & child_value) == 0u) {
            if (first_normal) {
                voxel_normal = normalize(child_normal);
            } else {
                voxel_normal = mix(voxel_normal, normalize(child_normal), 0.5);
            }
            voxel_normal = normalize(voxel_normal);
            first_normal = false;
            continue;
        }

        let is_leaf = ((leaves & child_value) != 0u);
        
        if ((parent.visited & child_value) == 0u) { // not visited yet
            let child_distance = voxel_distance(position, child_center, quarter_voxel_size);

            if (child_distance < minimum_distance) {
                minimum_distance = child_distance;

                hit = VoxelHit(is_leaf, next_pointer + child_offset, child_distance, child_center, half_voxel_size, 0u, child_mask | child_value, 0u, parent.normal);
            }

            if (is_leaf) {
                child_mask = (child_mask | child_value);
            }
        }

        if (is_leaf) {
            child_offset += 1u;
        } else {
            child_offset += 2u;
        }
    }

    if (parent.normal.x == 0.0 && parent.normal.y == 0.0 && parent.normal.z == 0.0) {
        hit.normal = voxel_normal;
    } else {
        hit.normal = mix(parent.normal, voxel_normal, mix(0.55, 0.75, hit.size));
    }

    return hit;
}

fn voxel_distance(position: vec3<f32>, center: vec3<f32>, half_size: f32) -> f32 {
    let shifted = abs((position - center) / half_size);

    return sqrt(pow(max(0.0, shifted.x - 1.0), 2.0) + pow(max(0.0, shifted.y - 1.0), 2.0) + pow(max(0.0, shifted.z - 1.0), 2.0)) * half_size;
}

fn simple_blinn_phong(color: vec4<f32>, normal: vec3<f32>) -> vec4<f32> {
    const specular_power = 2.0;
    const gloss = 0.9;

    let light_direction = normalize(vec3<f32>(0.8, 0.8, 1.0));
    let light_color = vec3<f32>(1.0, 1.0, 1.0);
    let n_dot_l = max(dot(normal, light_direction), 0.0);
    let view_direction = vec3<f32>(0.0, 0.0, 1.0); // set this based on camera
    let h = (light_direction - view_direction) / 2.;
    let specular = pow(dot(normal, h), specular_power) * gloss;

    return vec4<f32>(color.rgb * light_color * n_dot_l, 1.0) + specular;
}
