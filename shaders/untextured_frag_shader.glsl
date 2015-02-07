#version 130

in vec4 Color;

out vec4 out_color;


uniform vec4 color;

void main() {
  // This doesn't do gamma correction because it assumes the color is already gamma-corrected.
  out_color = color * Color;
}
