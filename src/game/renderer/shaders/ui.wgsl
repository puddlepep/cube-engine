
[[block]]
struct Uniforms {
    ui_proj: mat4x4<f32>;
};

[[group(0), binding(0)]]
var <uniform> uniforms: Uniforms;

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] normal: vec3<f32>;
    [[location(2)]] tex_coords: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] tex_coords: vec2<f32>;
    [[location(1)]] normal: vec3<f32>;
};

[[stage(vertex)]]
fn vertex(model: VertexInput) -> VertexOutput {

    var out: VertexOutput;
    out.tex_coords = model.tex_coords;
    out.clip_position = uniforms.ui_proj * vec4<f32>(model.position, 1.0);
    out.normal = model.normal;
    return out;

}

[[group(1), binding(0)]]
var ui_texture: texture_2d<f32>;

[[group(1), binding(1)]]
var ui_sampler: sampler;

[[stage(fragment)]]
fn fragment(in: VertexOutput) -> [[location(0)]] vec4<f32> {

    let tex_color = textureSample(ui_texture, ui_sampler, in.tex_coords);

    return tex_color;
}