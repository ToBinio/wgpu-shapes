use std::iter::once;

use wgpu::{CommandEncoder, PresentMode, RenderPipeline, TextureView};
use wgpu_noboiler::app::{AppCreator, AppData};
use wgpu_noboiler::render_pass::RenderPassCreator;
use winit::dpi::PhysicalSize;

use wgpu_shapes::shape_renderer::ShapeRenderer;

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
        .present_mode(PresentMode::Immediate)
        .run();
}

fn update(data: &AppData, state: &mut State) {
    state.rotation += 0.5 * data.delta_time as f32;
    state.shape_renderer.as_mut().unwrap().update_frame_offset((state.rotation.cos() * 50.0, 0.0));

    println!("{}", data.fps);
}

fn render(data: &AppData, state: &mut State, mut encoder: CommandEncoder, texture_view: TextureView) {
    let shape_renderer = state.shape_renderer.as_mut().unwrap();

    shape_renderer.clear();

    for x in -200..=200 {
        for y in -200..=200 {
            if x % 2 == 0 && y % 2 == 0 {
                shape_renderer.rect()
                    .width(20.0)
                    .height(20.0)
                    .pos((x as f32 * 25.0, y as f32 * 25.0))
                    .color(((x + 10) as f32 / 20.0, (y + 10) as f32 / 20.0, 0.0, 1.0))
                    .rotation(state.rotation);
            } else {
                shape_renderer.oval()
                    .width(20.0)
                    .height(20.0)
                    .pos((x as f32 * 25.0, y as f32 * 25.0))
                    .color((0.0, 0.0, 0.0, 1.0))
                    .detail(16);
            }
        }
    }

    shape_renderer.render(RenderPassCreator::new(&mut encoder, &texture_view).build(), &data.device);

    data.queue.submit(once(encoder.finish()));
}

fn init(data: &AppData, state: &mut State, _: &mut Vec<RenderPipeline>) {
    state.shape_renderer = Some(ShapeRenderer::new(&data.device, &data.config));
}

fn resize(_data: &AppData, state: &mut State, size: &PhysicalSize<u32>) {
    state.shape_renderer.as_mut().unwrap().update_frame_size((size.width as f32, size.height as f32));
}

