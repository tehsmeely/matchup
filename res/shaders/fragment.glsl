#version 140
in lowp vec2 uv;
in lowp vec4 jimty_color;
uniform sampler2D Texture;

out mediump vec4 fragColor;

void main() {
    fragColor = texture2D(Texture, uv);
}
