use std::iter::once;
use rand::{Rng, thread_rng};

use wgpu::{CommandEncoder, PowerPreference, PresentMode, RenderPipeline, TextureView};
use wgpu_noboiler::app::{AppCreator, AppData};
use winit::dpi::PhysicalSize;

use wgpu_shapes::shape::shapes::BasicShape;
use wgpu_shapes::shape_renderer::ShapeRenderer;

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
    .power_preference(PowerPreference::HighPerformance)
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

    for x in -400..400 {
        for y in -400..400 {
            shape_renderer
                .oval()
                .pos((x * 4) as f32, (y * 4) as f32)
                .scale(2.0, 2.0);
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
