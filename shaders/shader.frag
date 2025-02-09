#version 450

layout(location = 0) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

void main() {
    float tileSize = 1.0;
    vec3 pos = fragColor / tileSize;
    
    float xCell = floor(pos.x);
    float zCell = floor(pos.z);
    
    float checker = mod(xCell + zCell, 2.0);
    vec3 color = mix(vec3(1.0), vec3(0.0), checker);
    
    outColor = vec4(color, 1.0);
}
