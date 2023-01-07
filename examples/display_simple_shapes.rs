use std::iter::once;

use wgpu::{CommandEncoder, RenderPipeline, TextureView};
use wgpu_noboiler::app::{AppCreator, AppData};
use wgpu_noboiler::render_pass::RenderPassCreator;
use winit::dpi::PhysicalSize;

use wgpu_shapes::shape_renderer::ShapeRenderer;
use wgpu_shapes::shapes::BasicShape;

struct State {
    shape_renderer: Option<ShapeRenderer>,
    rotation: f32,
}

fn main() {
    AppCreator::new(State {
        shape_renderer: None,
        rotation: 0.0,
    })
        .render(render)
        .init(init)
        .update(update)
        .resize(resize)
        .run();
}

fn update(data: &AppData, state: &mut State) {
    state.rotation += 0.5 * data.delta_time as f32;
    state.shape_renderer.as_mut().unwrap().update_frame_offset((state.rotation.cos() * 50.0, 0.0));
}

fn render(data: &AppData, state: &mut State, mut encoder: CommandEncoder, texture_view: TextureView) {
    let shape_renderer = state.shape_renderer.as_mut().unwrap();

    shape_renderer.clear();

    shape_renderer.oval()
        .scale(500.0, 500.0)
        .color(0.0, 0.0, 0.0)
        .layer(0);

    shape_renderer.rect()
        .scale(400.0, 400.0)
        .color(1.0, 0.0, 0.0)
        .layer(1);

    shape_renderer.oval()
        .scale(300.0, 300.0)
        .color(0.0, 1.0, 0.0)
        .layer(4);

    shape_renderer.rect()
        .scale(200.0, 200.0)
        .color(0.0, 0.0, 1.0)
        .layer(30);

    shape_renderer.oval()
        .scale(100.0, 100.0)
        .color(1.0, 0.0, 1.0)
        .layer(100);

    shape_renderer.render(RenderPassCreator::new(&mut encoder, &texture_view), &data.device);

    data.queue.submit(once(encoder.finish()));
}

fn init(data: &AppData, state: &mut State, _: &mut Vec<RenderPipeline>) {
    state.shape_renderer = Some(ShapeRenderer::new(&data.device, &data.config));
}

fn resize(data: &AppData, state: &mut State, size: &PhysicalSize<u32>) {
    state.shape_renderer.as_mut().unwrap()
        .update_frame_size((size.width as f32, size.height as f32))
        .resize(&data.device, &data.config);
}

