#version 450

layout(location = 0) in vec4 fragColor;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec3 fragPos;

layout(push_constant) uniform Material {
    vec3 ambient;
    float dissolve;
    vec3 diffuse;
    float shininess;
    vec3 specular;
    float optical_density;
    int illum;
    float _pad1;
    float _pad2;
    float _pad3;
} mat;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 lightPos = vec3(0.0, 500.0, 500.0);

    vec3 norm = normalize(fragNormal);
    vec3 lightDir = normalize(lightPos - fragPos);
    
    vec3 color = mat.ambient;
    
    if (mat.illum >= 1) {
        float diff = max(dot(norm, lightDir), 0.0);
        color += diff * mat.diffuse * mat.dissolve;
    }
    
    if (mat.illum >= 2) {
        vec3 viewDir = normalize(-fragPos);
        vec3 reflectDir = reflect(-lightDir, norm);
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), mat.shininess);
        color += spec * mat.specular;
    }
    
    outColor = vec4(color, mat.dissolve);
}