#version 450

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} ubo;

layout(push_constant) uniform constants {
    mat4 model;
} pcs;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec2 inUv;

layout(location = 0) out vec3 fragPosition;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out vec2 fragUv;

void main() {
    gl_Position = ubo.proj * ubo.view * pcs.model * vec4(inPosition, 1.0);
    fragNormal = mat3(transpose(inverse(ubo.view * pcs.model))) * inNormal;
    fragPosition = vec3(ubo.view * pcs.model * vec4(inPosition, 1.0));
    fragUv = inUv;
}