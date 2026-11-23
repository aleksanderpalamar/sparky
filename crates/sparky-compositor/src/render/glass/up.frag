#version 100

//_DEFINES_

precision highp float;

varying vec2 v_coords;

uniform sampler2D tex;
uniform vec2 half_pixel;
uniform float offset;
uniform float alpha;
#if defined(DEBUG_FLAGS)
uniform float tint;
#endif

void main() {
    vec2 step = half_pixel * offset;
    vec4 sum = texture2D(tex, v_coords + vec2(-step.x * 2.0, 0.0));
    sum += texture2D(tex, v_coords + vec2(-step.x, step.y)) * 2.0;
    sum += texture2D(tex, v_coords + vec2(0.0, step.y * 2.0));
    sum += texture2D(tex, v_coords + vec2(step.x, step.y)) * 2.0;
    sum += texture2D(tex, v_coords + vec2(step.x * 2.0, 0.0));
    sum += texture2D(tex, v_coords + vec2(step.x, -step.y)) * 2.0;
    sum += texture2D(tex, v_coords + vec2(0.0, -step.y * 2.0));
    sum += texture2D(tex, v_coords + vec2(-step.x, -step.y)) * 2.0;
    gl_FragColor = sum / 12.0;
}
