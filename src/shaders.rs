use macroquad::material::{load_material, Material, MaterialParams};
use macroquad::miniquad::{
    BlendFactor, BlendState, BlendValue, Equation, PipelineParams, ShaderError, UniformType,
};
use std::fs;

const DEFAULT_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;

const DEFAULT_FRAGMENT: &str = r#"#version 100
varying lowp vec2 uv;
uniform sampler2D Texture;
void main() {
    gl_FragColor = texture2D(Texture, uv);
}"#;

const VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;

const FRAGMENT: &str = r#"#version 100
varying lowp vec2 uv;
uniform sampler2D Texture;
uniform lowp vec4 test_color;
void main() {
    gl_FragColor = test_color * texture2D(Texture, uv);
}"#;

fn load_material_basic(
    vertex_shader: &str,
    fragment_shader: &str,
) -> Result<Material, ShaderError> {
    load_material(
        vertex_shader,
        fragment_shader,
        MaterialParams {
            uniforms: vec![("test_color".to_string(), UniformType::Float4)],
            pipeline_params: PipelineParams {
                color_blend: Some(BlendState::new(
                    Equation::Add,
                    BlendFactor::Value(BlendValue::SourceAlpha),
                    BlendFactor::OneMinusValue(BlendValue::SourceAlpha),
                )),
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

pub fn glow_material() -> Material {
    load_material_basic(VERTEX, FRAGMENT).unwrap()
}

const VERTEX_OUTLINE: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
varying lowp vec2 uv;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    uv = texcoord;
}"#;

const FRAGMENT_OUTLINE: &str = r#"#version 100
varying lowp vec2 uv;
uniform sampler2D Texture;
uniform lowp vec4 test_color;
void main() {
    gl_FragColor = texture2D(Texture, uv);
}"#;

pub fn outline_material() -> Material {
    load_material_basic(VERTEX_OUTLINE, FRAGMENT_OUTLINE).unwrap()
}

pub fn material_from_file() -> Material {
    let vertex_txt = match fs::read_to_string("res/shaders/vertex.glsl") {
        Ok(s) => s,
        Err(e) => {
            println!(
                "Failed to load vertex shader from file, using default. Error: {:?}",
                e
            );
            DEFAULT_VERTEX.to_string()
        }
    };
    let fragment_txt = match fs::read_to_string("res/shaders/fragment.glsl") {
        Ok(s) => s,
        Err(e) => {
            println!(
                "Failed to load vertex shader from file, using default. Error: {:?}",
                e
            );
            DEFAULT_VERTEX.to_string()
        }
    };

    let maybe_mat = load_material_basic(&vertex_txt, &fragment_txt);
    match maybe_mat {
        Ok(mat) => mat,
        Err(e) => {
            println!(
                "Failed to load material from file, using default. Error: {:?}",
                e
            );
            load_material_basic(DEFAULT_VERTEX, DEFAULT_FRAGMENT).unwrap()
        }
    }
}
