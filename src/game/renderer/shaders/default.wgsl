
[[block]]
struct Uniforms {
    view_proj: mat4x4<f32>;

    light_color: vec3<f32>;
    light_dir: vec3<f32>;
};

[[group(0), binding(0)]]
var<uniform> uniforms: Uniforms;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] normal: vec3<f32>;

    [[location(2)]] light_dir: vec3<f32>;
    [[location(3)]] light_color: vec3<f32>;
};

[[stage(vertex)]]
fn vertex(model: VertexInput) -> VertexOutput {

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = uniforms.view_proj * vec4<f32>(model.position, 1.0);
    out.normal = model.normal;
    out.light_dir = uniforms.light_dir;
    out.light_color = uniforms.light_color;
    return out;

}

[[group(1), binding(0)]]
var chunk_texture: texture_2d<f32>;

[[group(1), binding(1)]]
var chunk_sampler: sampler;

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    let tex_color = textureSample(chunk_texture, chunk_sampler, in.tex_coords);
    let ambient_strength = 0.1;
    let ambient_color = in.light_color * ambient_strength;

    let diffuse_strength = max(dot(in.light_dir, in.normal), 0.0);
    let diffuse_color = in.light_color * diffuse_strength;

    let result = (ambient_color + diffuse_color) * tex_color.rgb;

    return vec4<f32>(result, 1.0);
}