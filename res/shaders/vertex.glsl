#version 140

in vec3 position;
in vec2 texcoord;
in vec4 projection;
in vec4 model;


out vec2 uv;
out vec4 jimty_color;

void main()
{
    jimty_color = vec4(1.5, 1.5, 1.5, 1.0);
    gl_Position = projection * model * vec4(position, 1.0);
    uv = texcoord;
}
