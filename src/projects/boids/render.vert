precision mediump float;

attribute float a_index;
uniform sampler2D u_input;
uniform vec2 u_dimensions;
uniform float u_aspect;

vec4 getValueFrom2DTextureAs1DArray(sampler2D tex, vec2 dimensions, float index) {
    float y = floor(index / dimensions.x);
    float x = mod(index, dimensions.x);
    vec2 texcoord = (vec2(x, y) + 0.5) / dimensions;
    return texture2D(tex, texcoord);
}

void main() {
    float index = floor(a_index / 3.0);
    vec4 data = getValueFrom2DTextureAs1DArray(u_input, u_dimensions, index);
    vec2 dir = normalize(data.zw);
    mat2 rotation_matrix = mat2(vec2(dir.x, -dir.y), vec2(dir.y, dir.x));

    vec2 VERTS[3];
    VERTS[0] = vec2(0.0, 0.5);
    VERTS[1] = vec2(-0.25, -0.25);
    VERTS[2] = vec2(0.25, -0.25);
    float SCALE = 0.1;
    vec2 vertex;
    if (mod(a_index, 3.0) == 0.0) {
        vertex = VERTS[0];
    } else if (mod(a_index, 3.0) == 1.0) {
        vertex = VERTS[1];
    } else {
        vertex = VERTS[2];
    }

    gl_Position = vec4((data.xy + SCALE * (rotation_matrix * vertex)) * vec2(u_aspect, 1.0), 0.0, 1.0);
}
