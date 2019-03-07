#version 450

layout(set = 0, binding = 0) uniform Data {
    uniform mat4 transform;
} uniforms;

layout(location = 0) in vec2 tl;
layout(location = 1) in vec2 br;
layout(location = 2) in vec2 tex_tl;
layout(location = 3) in vec2 tex_br;
layout(location = 4) in vec4 color;

layout(location = 0) out vec2 f_tex_pos;
layout(location = 1) out vec4 f_color;

void main() {
    vec2 pos;

    switch (gl_VertexIndex) {
        case 0: // bottom left 
            pos = vec2(tl.x, br.y);
            f_tex_pos = vec2(tex_tl.x, tex_br.y);
            break;
        case 1: // top left
            pos = tl;
            f_tex_pos = tex_tl;
            break;
        case 2: // bottom right
            pos = br;
            f_tex_pos = tex_br;
            break;
        case 3: // top right
            pos = vec2(br.x, tl.y);
            f_tex_pos = vec2(tex_br.x, tex_tl.y);
            break;
    }

    f_color = color;
    gl_Position = uniforms.transform * vec4(pos, 0.0, 1.0);
}
