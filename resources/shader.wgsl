// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput{
    @location(1) position: vec2<f32>,
    @location(2) scale: vec2<f32>,
    @location(3) color: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instace: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;
    out.color = instace.color;

    var xPos: f32;
    xPos = (model.position.x * instace.scale.x) + instace.position.x;

    var yPos: f32;
    yPos = (model.position.y * instace.scale.y) + instace.position.y;

    out.clip_position = vec4<f32>(xPos,yPos,0.0, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}