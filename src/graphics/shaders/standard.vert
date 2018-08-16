#version 330 core

const int BATCH_SIZE = 150;

layout (location = 0) in vec3 Position;
layout (location = 1) in vec2 UV;
layout (location = 2) in vec4 TexPosition;
layout (location = 3) in mat4 TransformMatrix;

out vec2 UVCoord;
out vec2 SourcePosition;
out vec2 SourceSize;

void main() {
    gl_Position = TransformMatrix * vec4(Position, 1.0);

    UVCoord = UV;
    SourcePosition = TexPosition.xy;
    SourceSize = TexPosition.zw;
}
