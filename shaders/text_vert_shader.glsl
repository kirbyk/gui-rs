#version 130

in vec2 position;
in vec2 texcoord;
in vec4 color;

uniform mat4 matrix;

out vec2 Texcoord;
out vec4 Color;

void main() {
  gl_Position = matrix * vec4(position, 0.0, 1.0);
  Texcoord = texcoord;
  Color = color;
}
