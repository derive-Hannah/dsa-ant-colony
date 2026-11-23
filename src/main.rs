use macroquad::prelude::*;

mod hex_grid;
use hex_grid::HexGrid;

#[macroquad::main("Ant Simulation")]
async fn main() {
    let mut cam = Camera2D {
        target: vec2(0., 0.0),
        ..Default::default()
    };

    let mut cam_zoom = 0.005;

    loop {
        // INPUT HANDLING
        // Pan camera
        if is_mouse_button_down(MouseButton::Right) {
            let delta = mouse_delta_position();
            cam.target.x += delta.x / cam_zoom * screen_width() / screen_height();
            cam.target.y += delta.y / cam_zoom;
        }
        // Zoom camera
        // Pan the camera to the mouse, zoom in, then pan away
        {
            let mouse_pos = mouse_position_local();
            cam.target.x += mouse_pos.x / cam_zoom * screen_width() / screen_height();
            cam.target.y += mouse_pos.y / cam_zoom;
            cam_zoom *= 2.0_f32.powf(mouse_wheel().1/1000.0);
            cam.target.x -= mouse_pos.x / cam_zoom * screen_width() / screen_height();
            cam.target.y -= mouse_pos.y / cam_zoom;
        }

        // UPDATE

        // DRAW
        clear_background(BLACK);

        cam.zoom = vec2(cam_zoom * screen_height() / screen_width(), cam_zoom);
        set_camera(&cam);

        draw_hexagon(0., 0., 10., 1., true, DARKGREEN, GREEN);

        next_frame().await
    }
}
