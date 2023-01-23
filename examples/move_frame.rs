use std::iter::once;

use wgpu::{CommandEncoder, RenderPipeline, TextureView};
use wgpu_noboiler::app::{AppCreator, AppData};
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};

use wgpu_shapes::render::shape_renderer::ShapeRenderer;
use wgpu_shapes::shape::shapes::BasicShape;

struct State {
    shape_renderer: Option<ShapeRenderer>,
    dragging: bool,
    last_drag_pos: (f32, f32),
}

fn main() {
    AppCreator::new(State {
        shape_renderer: None,
        dragging: false,
        last_drag_pos: (0.0, 0.0),
    })
        .render(render)
        .init(init)
        .update(update)
        .resize(resize)
        .window_event(event)
        .run();
}

fn update(_data: &AppData, _state: &mut State) {}

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

    shape_renderer.image()
        .scale(200.0, 200.0)
        .layer(105);

    shape_renderer.render(&mut encoder, &texture_view, &data.device);

    data.queue.submit(once(encoder.finish()));
}

fn init(data: &AppData, state: &mut State, _: &mut Vec<RenderPipeline>) {
    state.shape_renderer = Some(ShapeRenderer::new(&data.device, &data.config));
    state.shape_renderer.as_mut().unwrap().add_texture_from_bytes(include_bytes!("img.png"), &data.device, &data.queue)
        .add_texture_from_bytes(include_bytes!("img2.png"), &data.device, &data.queue)
        .add_texture_from_bytes(include_bytes!("img.png"), &data.device, &data.queue);
}

fn resize(data: &AppData, state: &mut State, size: &PhysicalSize<u32>) {
    state.shape_renderer.as_mut().unwrap()
        .set_frame_size((size.width as f32, size.height as f32))
        .resize(&data.device, &data.config);
}

fn event(_app_data: &AppData, app_state: &mut State, window_event: &WindowEvent) {
    match window_event {
        WindowEvent::CursorMoved { position, .. } => {
            if app_state.dragging {
                let renderer = app_state.shape_renderer.as_mut().unwrap();

                let mut offset = renderer.frame_offset();

                if app_state.last_drag_pos != (-1.0, -1.0) {
                    offset.0 -= app_state.last_drag_pos.0 - position.x as f32;
                    offset.1 += app_state.last_drag_pos.1 - position.y as f32;
                }

                renderer.set_frame_offset(offset);
                app_state.last_drag_pos = (position.x as f32, position.y as f32);
            }
        }
        WindowEvent::MouseInput { state, button, .. } => {
            if button != &MouseButton::Left { return; }
            app_state.dragging = state == &ElementState::Pressed;
            app_state.last_drag_pos = (-1.0, -1.0);
        }
        _ => {}
    }
}

