use std::f32::consts::PI;
use std::iter::once;

use wgpu::{CommandEncoder, PresentMode, RenderPipeline, TextureView};
use wgpu_noboiler::app::{AppCreator, AppData};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};

use wgpu_shapes::render::shape_renderer::ShapeRenderer;
use wgpu_shapes::shape::shapes::BasicShape;

struct State {
    shape_renderer: Option<ShapeRenderer>,
    count: i32,
}

fn main() {
    AppCreator::new(State {
        shape_renderer: None,
        count: 0,
    })
    .render(render)
    .init(init)
    .resize(resize)
    .update(|app_data: &AppData, state: &mut State| {
        state.count += 1;

        if state.count >= 60 {
            println!("{}", app_data.fps);
            state.count = 0;
        }
    })
    .present_mode(PresentMode::Immediate)
    .run();
}

fn render(
    data: &AppData,
    state: &mut State,
    mut encoder: CommandEncoder,
    texture_view: TextureView,
) {
    let shape_renderer = state.shape_renderer.as_mut().unwrap();

    shape_renderer.clear();

    for x in -200..200 {
        for y in -200..200 {
            shape_renderer
                .image(0)
                .pos((x * 2) as f32, (y * 2) as f32)
                .scale(1.0, 1.0);
        }
    }

    shape_renderer.render(&mut encoder, &texture_view, &data.device);

    data.queue.submit(once(encoder.finish()));
}

fn init(data: &AppData, state: &mut State, _: &mut Vec<RenderPipeline>) {
    state.shape_renderer = Some(ShapeRenderer::new(&data.device, &data.config));
    state
        .shape_renderer
        .as_mut()
        .unwrap()
        .add_textures_from_bytes(&vec![include_bytes!("img.png")], &data.device, &data.queue);
}

fn resize(data: &AppData, state: &mut State, size: &PhysicalSize<u32>) {
    state
        .shape_renderer
        .as_mut()
        .unwrap()
        .set_frame_size((size.width as f32, size.height as f32))
        .resize(&data.device, &data.config);
}
