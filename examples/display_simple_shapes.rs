use std::iter::once;

use wgpu::{CommandEncoder, RenderPipeline, TextureView};
use wgpu_noboiler::app::{AppCreator, AppData};
use wgpu_noboiler::render_pass::RenderPassCreator;
use winit::dpi::PhysicalSize;

use wgpu_shapes::shape_renderer::ShapeRenderer;

struct State {
    shape_renderer: Option<ShapeRenderer>,
}

fn main() {
    AppCreator::new(State {
        shape_renderer: None
    })
        .render(render)
        .init(init)
        .resize(resize)
        .run();
}

fn render(data: &AppData, state: &mut State, mut encoder: CommandEncoder, texture_view: TextureView) {
    let shape_renderer = state.shape_renderer.as_mut().unwrap();

    shape_renderer.clear();

    shape_renderer.rect()
        .color((0.0, 1.0, 1.0, 1.0));

    shape_renderer.rect()
        .pos((0.0, 50.0))
        .width(100.0)
        .color((0.0, 1.0, 0.0, 1.0));

    shape_renderer.rect()
        .pos((0.0, -50.0))
        .width(100.0)
        .color((0.0, 1.0, 0.0, 1.0));


    shape_renderer.render(RenderPassCreator::new(&mut encoder, &texture_view).build(), &data.device);

    data.queue.submit(once(encoder.finish()));
}

fn init(data: &AppData, state: &mut State, _: &mut Vec<RenderPipeline>) {
    state.shape_renderer = Some(ShapeRenderer::new(&data.device, &data.config));
}

fn resize(_data: &AppData, state: &mut State, size: &PhysicalSize<u32>) {
    state.shape_renderer.as_mut().unwrap().frame_size((size.width as f32, size.height as f32));
}

