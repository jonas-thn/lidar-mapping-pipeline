struct CameraUniform {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    proj: mat4x4<f32>,
}
@group(0) @binding(0) var<uniform> camera: CameraUniform;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
    @location(1) uv: vec2<f32>, 
};

struct QuadInput {
    @location(0) position: vec2<f32>,
}
struct InstanceInput {
    @location(1) instance_pos: vec3<f32>,
    @location(2) instance_color: vec3<f32>,
}

@vertex 
fn vs_lidar(quad: QuadInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    let camera_right = vec3<f32>(camera.view[0][0], camera.view[1][0], camera.view[2][0]);
    let camera_up    = vec3<f32>(camera.view[0][1], camera.view[1][1], camera.view[2][1]);

    let radius = 20.0;

    /*scales & reorients every corner of quad (away from middle) */
    let world_pos = instance.instance_pos 
                  + camera_right * quad.position.x * radius
                  + camera_up    * quad.position.y * radius;

    out.clip_position = camera.proj * camera.view * vec4<f32>(world_pos, 1.0);
    out.color = instance.instance_color;
    out.uv = quad.position;
    return out;
}

@fragment 
fn fs_lidar(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.uv);
    if dist > 1.0 {
        discard;
    }
    return vec4<f32>(in.color, 1.0);
}

struct GridInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
}

@vertex
fn vs_grid(model: GridInput) -> VertexOutput {
    var out: VertexOutput;

    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.color = model.color;
    out.uv = vec2<f32>(0.0, 0.0);

    return out;
}

@fragment 
fn fs_grid(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}