use std::f32::consts::PI;
use std::iter::once;

use wgpu::{Color, CommandEncoder, RenderPipeline, TextureView};
use wgpu_noboiler::app::{AppCreator, AppData};

use wgpu_shapes::render::shape_renderer::ShapeRenderer;
use wgpu_shapes::shape::shapes::BasicShape;

struct State {
    shape_renderer: Option<ShapeRenderer>,
}

fn main() {
    AppCreator::new(State {
        shape_renderer: None,
    })
    .render(render)
    .init(init)
    .resizable(false)
    .title("Display simple shapes")
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

    shape_renderer.oval().pos(25.0, 50.0).layer(1);

    shape_renderer
        .oval()
        .pos(25.0, 50.0)
        .color(0.1, 0.5, 1.0)
        .scale(200.0, 50.0)
        .segment_count(6)
        .rotation(PI / 4.0)
        .layer(0);

    shape_renderer
        .oval()
        .pos(25.0, 50.0)
        .color(0.1, 0.5, 1.0)
        .scale(200.0, 50.0)
        .segment_count(6)
        .rotation(-PI / 4.0)
        .layer(0);

    for i in (1..6).rev() {
        let color = i as f32 / 5.0;

        shape_renderer
            .rect()
            .pos(-200.0, -100.0)
            .scale((i * 50) as f32, (i * 50) as f32)
            .color(color, color, color)
            .layer(5 - i as u16);
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
        .background_color(Color::BLACK);
}
