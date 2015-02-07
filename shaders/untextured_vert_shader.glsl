#version 130

in vec2 position;
in vec4 color;

out vec4 Color;

uniform mat4 modelViewMatrix;
uniform mat4 projMatrix;

void main() {
  gl_Position = projMatrix * modelViewMatrix * vec4(position, 0.0, 1.0);
  Color = color;
}
