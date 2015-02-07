#version 130

in vec2 Texcoord;

out float out_color;

uniform sampler2D tex;

void main() {
  vec4 tex_color = texture(tex, Texcoord);
  out_color = tex_color.r;
}
