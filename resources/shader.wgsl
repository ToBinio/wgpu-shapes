// Vertex shader

@group(0) @binding(0)
var<uniform> frameSize : vec2<f32>;

@group(0) @binding(1)
var<uniform> frameOffset : vec2<f32>;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput{
    @location(1) position: vec2<f32>,
    @location(2) scale: vec2<f32>,
    @location(3) rotation: f32,
    @location(4) color: vec3<f32>,
    @location(5) layer: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;
    out.color = instance.color;

    var xScale = instance.scale.x / 2.0;
    var yScale = instance.scale.y / 2.0;

    var xLocation = model.position.x * xScale;
    var yLocation = model.position.y * yScale;
    var zLocation =  0.9 - (f32(instance.layer) / 75000.0);

    var xPos = ((xLocation * cos(instance.rotation) - yLocation * sin(instance.rotation)) + instance.position.x + frameOffset.x) / frameSize.x * 2.0;
    var yPos = ((xLocation * sin(instance.rotation) + yLocation * cos(instance.rotation)) + instance.position.y + frameOffset.y) / frameSize.y * 2.0;

    out.clip_position = vec4<f32>(xPos,yPos,zLocation, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}