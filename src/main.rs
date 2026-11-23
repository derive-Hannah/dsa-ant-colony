#![allow(clippy::type_complexity)]
use std::cell::{Ref, RefCell, RefMut};

use macroquad::prelude::*;

mod hex_grid;
use hex_grid::HexGrid;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Land {
    #[default]
    Empty,
    Nest(f32),
    Food(f32),
}

#[derive(Debug, Default, Clone, PartialEq)]
struct Cell {
    land: Land,
    ants: Vec<Ant>,
    pheremone_found: f64,
    pheremone_searching: f64,
}

#[derive(Debug, Clone, PartialEq)]
struct Ant {
    pos: Vec2,
}

const HEX_RADIUS: f32 = 100.0;
const COS_30: f32 = 0.86602540378443864676372317075294;

// Helper functions for hex coordinate to pixel coordinate conversion
fn hex_to_pixel(pos: IVec2) -> Vec2 {
    vec2(
        pos.x as f32 * HEX_RADIUS * COS_30 * 2.0 + pos.y as f32 * HEX_RADIUS * COS_30,
        pos.y as f32 * HEX_RADIUS * 1.5,
    )
}

fn pixel_to_hex(pos: Vec2) -> IVec2 {
    let mut hex_y = pos.y / (HEX_RADIUS * 1.5);
    let mut hex_x = (pos.x / (HEX_RADIUS * COS_30) - hex_y) / 2.0;
    // Axial rounding algorithm courtsey of Jacob Rus
    // https://observablehq.com/@jrus/hexround
    let rounded = vec2(hex_x.round(), hex_y.round());
    hex_x -= rounded.x;
    hex_y -= rounded.y;
    if hex_x.abs() >= hex_y.abs() {
        ivec2(
            (rounded.x + (hex_x + 0.5 * hex_y).round()) as i32,
            rounded.y as i32,
        )
    } else {
        ivec2(
            rounded.x as i32,
            (rounded.y + (hex_y + 0.5 * hex_x).round()) as i32,
        )
    }
}

// Extract logic out to a function
// Inline it so there is no performance cost for calling in a hotloop
#[inline]
fn update_cell(
    cell: RefMut<Cell>,
    top_left: Option<RefMut<Cell>>,
    top_right: Option<RefMut<Cell>>,
    left: Option<RefMut<Cell>>,
    right: Option<RefMut<Cell>>,
    bottom_left: Option<RefMut<Cell>>,
    bottom_right: Option<RefMut<Cell>>,
) {
    // Be VERY mindful of performance here
    // This function is run for every cell, every frame.
}

// Same considerations as above
#[inline]
fn draw_cell(cell: Ref<Cell>, x: i32, y: i32) {
    let graphical_pos = hex_to_pixel(ivec2(x, y));
    draw_hexagon(
        graphical_pos.x,
        graphical_pos.y,
        HEX_RADIUS - HEX_RADIUS / 10.0 / 2.0,
        HEX_RADIUS / 10.0,
        true,
        DARKGREEN,
        GREEN,
    );

    match cell.land {
        Land::Empty => {}
        Land::Nest(food) => {
            draw_text(
                "Nest",
                graphical_pos.x - HEX_RADIUS / 1.7,
                graphical_pos.y - HEX_RADIUS / 4.0,
                HEX_RADIUS / 1.5,
                WHITE,
            );
            draw_text(
                food.to_string(),
                graphical_pos.x - HEX_RADIUS / 5.0,
                graphical_pos.y + HEX_RADIUS / 4.0,
                HEX_RADIUS / 1.5,
                WHITE,
            );
        }
        Land::Food(_) => todo!(),
    }
}

#[macroquad::main("Ant Simulation")]
async fn main() {
    let mut cam = Camera2D {
        target: vec2(0., 0.0),
        ..Default::default()
    };

    let mut cam_zoom = 0.0005;

    let mut grid =
        HexGrid::<RefCell<Cell>>::from_bounds_hexagon(RefCell::new(Cell::default()), -1, 0, 1);
    grid.set(
        0,
        0,
        RefCell::new(Cell {
            land: Land::Nest(0.0),
            ..Default::default()
        }),
    );

    loop {
        // INPUT HANDLING
        {
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
                cam_zoom *= 2.0_f32.powf(mouse_wheel().1 / 1000.0);
                cam.target.x -= mouse_pos.x / cam_zoom * screen_width() / screen_height();
                cam.target.y -= mouse_pos.y / cam_zoom;
            }
            // Add tiles
            if is_mouse_button_pressed(MouseButton::Left) {}
        }

        // UPDATE
        {}

        // DRAW pt. 1
        {
            clear_background(BLACK);

            cam.zoom = vec2(cam_zoom * screen_height() / screen_width(), cam_zoom);
            set_camera(&cam);
        }

        // ITERATE CELLS
        {
            for y in grid.y_bounds() {
                let above = grid.get_row(y - 1);
                let current = grid.get_row(y);
                let below = grid.get_row(y + 1);

                if let Some((mut x, cur_row)) = current {
                    let mut prev = None;
                    let mut row_iter = cur_row.0.iter().chain(cur_row.1).peekable();
                    while let Some(cell) = row_iter.next() {
                        let (top_left, top_right) = above
                            .map(|(above_x_offset, above_row)| {
                                let mut above_row_iter = above_row
                                    .0
                                    .iter()
                                    .chain(above_row.1.iter())
                                    .skip((x - above_x_offset) as usize);
                                if above_x_offset > x + 1 {
                                    (None, None)
                                } else if above_x_offset == x + 1 {
                                    (None, above_row_iter.next())
                                } else {
                                    (above_row_iter.next(), above_row_iter.next())
                                }
                            })
                            .unwrap_or((None, None));
                        let left = prev;
                        let right = row_iter.peek().copied();
                        let (bottom_left, bottom_right) = below
                            .map(|(below_x_offset, below_row)| {
                                let mut below_row_iter = below_row
                                    .0
                                    .iter()
                                    .chain(below_row.1.iter())
                                    .skip((x - below_x_offset) as usize);
                                if below_x_offset > x {
                                    (None, None)
                                } else if below_x_offset == x {
                                    (None, below_row_iter.next())
                                } else {
                                    (below_row_iter.next(), below_row_iter.next())
                                }
                            })
                            .unwrap_or((None, None));

                        {
                            // All operations on a cell are performed in the same loop.
                            // This ensures we don't have to fetch the entire grid
                            // from memory twice.

                            update_cell(
                                cell.borrow_mut(),
                                top_left.map(|c: &RefCell<Cell>| c.borrow_mut()),
                                top_right.map(|c: &RefCell<Cell>| c.borrow_mut()),
                                left.map(|c: &RefCell<Cell>| c.borrow_mut()),
                                right.map(|c: &RefCell<Cell>| c.borrow_mut()),
                                bottom_left.map(|c: &RefCell<Cell>| c.borrow_mut()),
                                bottom_right.map(|c: &RefCell<Cell>| c.borrow_mut()),
                            );

                            draw_cell(cell.borrow(), x, y);
                        }

                        prev = Some(cell);
                        x += 1;
                    }
                }
            }
        }

        // DRAW pt 2.
        {
            // Highlight hovered cell
            let hex_pos = pixel_to_hex(cam.screen_to_world(Vec2::from(mouse_position())));
            let graphical_pos = hex_to_pixel(hex_pos);
            draw_hexagon(
                graphical_pos.x,
                graphical_pos.y,
                HEX_RADIUS - HEX_RADIUS / 10.0 / 2.0,
                HEX_RADIUS / 10.0,
                true,
                WHITE,
                Color::from_rgba(0, 0, 0, 100),
            );
        }

        next_frame().await
    }
}
