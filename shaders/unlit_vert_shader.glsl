#version 130

in vec2 position;
in vec2 texcoord;

uniform mat4 modelViewMatrix;
uniform mat4 projMatrix;

out vec2 Texcoord;

void main() {
  gl_Position = projMatrix * modelViewMatrix * vec4(position, 0.0, 1.0);
  Texcoord = texcoord;
}
