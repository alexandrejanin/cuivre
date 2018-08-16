#version 330 core

in vec2 UVCoord;
in vec2 SourcePosition;
in vec2 SourceSize;

uniform sampler2D Tex;

out vec4 Color;

void main() {
    vec4 color =  texture(Tex, SourcePosition + SourceSize * UVCoord);

    if (color.a <= 0.01) {
        discard;
    } else {
        Color = color;
    }
}
