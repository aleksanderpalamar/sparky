#version 100

//_DEFINES_

precision highp float;

varying vec2 v_coords;

uniform sampler2D tex;
uniform float alpha;
uniform vec4 rect;
uniform vec2 size;
uniform float radius;
uniform float grain;
#if defined(DEBUG_FLAGS)
uniform float tint;
#endif

float speckle(vec2 point) {
    return fract(sin(dot(point, vec2(12.9898, 78.233))) * 43758.5453);
}

void main() {
    vec2 point = (v_coords - rect.xy) / rect.zw * size;
    vec2 corner = abs(point - size * 0.5) - (size * 0.5 - vec2(radius));
    float distance = length(max(corner, 0.0)) - radius;
    float coverage = clamp(0.5 - distance, 0.0, 1.0);
    vec4 color = texture2D(tex, v_coords);
    color.rgb += (speckle(v_coords) - 0.5) * grain * color.a;
    gl_FragColor = color * alpha * coverage;
}
