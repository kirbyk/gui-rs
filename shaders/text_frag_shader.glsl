#version 130

in vec2 Texcoord;
in vec4 Color;

out vec4 out_color;

uniform sampler2D tex;

void main() {
  vec4 tex_color = texture(tex, Texcoord);
  out_color = vec4(Color.rgb, tex_color.r * Color.a);
}
