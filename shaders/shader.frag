#version 450

layout(set = 0, binding = 1) uniform sampler2D tex[512];

layout(location = 0) in vec3 fragPosition;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragUv;

layout(push_constant) uniform Material {
    layout(offset = 64)
    vec3 ambient;
    float dissolve;
    vec3 diffuse;
    float shininess;
    vec3 specular;
    float optical_density;
    int illum;
    uint tex_diffuse;
    uint tex_ambient;
    uint tex_specular;
} mat;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 lightPos = vec3(0.0, 500.0, 500.0);
    vec3 norm = normalize(fragNormal);
    vec3 lightDir = normalize(lightPos - fragPosition);

    vec3 ambient_color  = texture(tex[mat.tex_ambient],  fragUv).rgb * mat.ambient;
    vec3 diffuse_color  = texture(tex[mat.tex_diffuse],  fragUv).rgb * mat.diffuse;
    vec3 specular_color = texture(tex[mat.tex_specular], fragUv).rgb * mat.specular;

    vec3 color = ambient_color;

    if (mat.illum >= 1) {
        float diff = max(dot(norm, lightDir), 0.0);
        color += diff * diffuse_color * mat.dissolve;
    }

    if (mat.illum >= 2) {
        vec3 viewDir = normalize(-fragPosition);
        vec3 reflectDir = reflect(-lightDir, norm);
        float spec = pow(max(dot(viewDir, reflectDir), 0.0), max(mat.shininess, 1.0));
        color += spec * specular_color;
    }

    outColor = vec4(color, mat.dissolve);
}