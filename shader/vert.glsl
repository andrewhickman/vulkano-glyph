#version 450

layout(set = 0, binding = 0) uniform Data {
    uniform mat4 transform;
} uniforms;

layout(location = 0) in vec2 tl;
layout(location = 1) in vec2 br;
layout(location = 2) in vec2 tex_tl;
layout(location = 3) in vec2 tex_br;
layout(location = 4) in vec4 color;
layout(location = 5) in float z;

layout(location = 0) out vec2 f_tex_pos;
layout(location = 1) out vec4 f_color;

// generate positional data based on vertex ID
void main() {
    vec2 pos;

    //vec2 tl = vec2(-0.798, -0.816);
    //vec2 br = vec2(-0.788, 0.81);
    //vec2 tex_tl = vec2(0.00390625, 0.00390625);
    //vec2 tex_br = vec2(0.0234375, 0.015625);

    switch (gl_VertexIndex) {
        case 0: // top left
            pos = tl;
            f_tex_pos = tex_tl;
            break;
        case 1: // top right
            pos = vec2(br.x, tl.x);
            f_tex_pos = vec2(tex_br.x, tex_tl.y);
            break;
        case 2: // bottom right
            pos = br;
            f_tex_pos = tex_br;
            break;
        case 3: // bottom left 
            pos = vec2(tl.x, br.y);
            f_tex_pos = vec2(tex_tl.x, tex_br.y);
            break;
    }

    f_color = color;
    gl_Position = uniforms.transform * vec4(pos, z, 1.0);
}
