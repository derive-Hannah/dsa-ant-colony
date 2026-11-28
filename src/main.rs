#![allow(clippy::type_complexity)]
use std::{cell::{Ref, RefCell, RefMut}, collections::VecDeque, iter};

use macroquad::prelude::*;

mod hex_grid;
use hex_grid::HexGrid;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
enum Land {
    #[default]
    Empty,
    Nest(u32),
    Food(u32),
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
    carrying_food: bool,
    updated_this_frame: bool,
    moving_right: bool,
    target: Option<IVec2>,
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

fn mouse_hex_position(cam: &Camera2D) -> IVec2 {
    pixel_to_hex(cam.screen_to_world(Vec2::from(mouse_position())))
}

fn hex_distance(diff: IVec2) -> i32 {
    (diff.x.abs() + diff.y.abs() + (diff.x + diff.y).abs()) / 2
}

fn movement_heuristic(from_pos: IVec2, to_pos: IVec2, carrying: bool, target: &Cell) -> f64 {
    // Pull of nest when carrying food
    const TO_NEST_WEIGHT: f64 = 3.0;
    // Repel of nest when not carrying food
    const AWAY_NEST_WEIGHT: f64 = 0.25;
    // Pheremone influence
    const FOUND_PHEREMONE_WEIGHT: f64 = 5.0;
    const SEARCHING_PHEREMONE_WEIGHT: f64 = 1.0;


    let mut score = 0.0;

    let nest_aproachness = (hex_distance(from_pos) - hex_distance(to_pos)) as f64;
    if carrying {
        score += TO_NEST_WEIGHT * nest_aproachness;
    } else {
        score -= AWAY_NEST_WEIGHT * nest_aproachness;
    }

    if !carrying {
        score += target.pheremone_found * FOUND_PHEREMONE_WEIGHT;
        score -= target.pheremone_searching * SEARCHING_PHEREMONE_WEIGHT;
    }

    score
}

fn pick_neighbor(from_pos: IVec2, carrying: bool, neighbors: &[(IVec2, Option<RefMut<Cell>>); 6]) -> Option<usize> {
    // Always prioritize food-containing cells when not carrying food
    if !carrying {
        if let Some(i) = neighbors.iter().position(|(_, n)| {
            matches!(n.as_deref().map(|c| c.land), Some(Land::Food(f)) if f > 0)
        }) {
            return Some(i);
        }
    }

    let mut weights = [0.0; 6];
    let mut total = 0.0;
    for (i, (offset, n)) in neighbors.iter().enumerate() {
        if let Some(target) = n.as_deref() {
            let w = movement_heuristic(from_pos, from_pos + *offset, carrying, target).exp() as f32;
            weights[i] = w;
            total += w;
        }
    }
    if total <= 0.0 {
        return None;
    }

    let mut roll = rand::gen_range(0.0, total);
    for (i, w) in weights.iter().enumerate() {
        roll -= *w;
        if roll < 0.0 {
            return Some(i);
        }
    }
    // in case of floating point inprecision
    weights.iter().rposition(|w| *w > 0.0)
}

// Extract logic out to a function
fn update_cell(
    delta: f32,
    cell_pos: IVec2,
    mut cell: RefMut<Cell>,
    top_left: Option<RefMut<Cell>>,
    top_right: Option<RefMut<Cell>>,
    left: Option<RefMut<Cell>>,
    right: Option<RefMut<Cell>>,
    bottom_left: Option<RefMut<Cell>>,
    bottom_right: Option<RefMut<Cell>>,
) {
    // Be VERY mindful of performance here
    // This function is run for every cell, every frame.

    const ANT_SPEED: f32 = 100.0;

    let cell: &mut Cell = &mut cell;

    let mut neighbors = [
        (ivec2(0, -1), top_left),
        (ivec2(1, -1), top_right),
        (ivec2(-1, 0), left),
        (ivec2(1, 0), right),
        (ivec2(-1, 1), bottom_left),
        (ivec2(0, 1), bottom_right),
    ];

    // Pheremone spread
    let reduced_pheremone_found = cell.pheremone_found * (1.0 - 0.5f64.powf(delta as f64 * 0.5));
    let reduced_pheremone_searching = cell.pheremone_searching * (1.0 - 0.5f64.powf(delta as f64));
    cell.pheremone_found -= reduced_pheremone_found;
    cell.pheremone_searching -= reduced_pheremone_searching;

    for (_, cell) in &mut neighbors {
        let Some(cell) = cell else { continue };
        cell.pheremone_found += reduced_pheremone_found / 6.0;
        cell.pheremone_searching += reduced_pheremone_searching / 6.0;
    }

    let ants = std::mem::take(&mut cell.ants);

    for mut ant in ants {
        if ant.updated_this_frame {
            ant.updated_this_frame = false;
            cell.ants.push(ant);
            continue;
        }

        if ant.carrying_food {
            if let Land::Nest(food) = &mut cell.land {
                *food += 1;
                ant.carrying_food = false;
            }
        } else if let Land::Food(food) = &mut cell.land {
            *food -= 1;
            ant.carrying_food = true;
            if *food == 0 {
                cell.land = Land::Empty;
            }
        }

        if ant.carrying_food {
            cell.pheremone_found += delta as f64;
        }
        cell.pheremone_searching += 0.8 * delta as f64;

        // Choose target if none is selected and old target reached
        if ant.target.is_none_or(|t| hex_to_pixel(t).distance(ant.pos) < HEX_RADIUS / 2.0) {
            ant.target = pick_neighbor(cell_pos, ant.carrying_food, &neighbors)
                .map(|i| cell_pos + neighbors[i].0);
        }

        // Move towards target
        if let Some(target) = ant.target {
            let towards_target = hex_to_pixel(target) - ant.pos;
            ant.pos += towards_target.normalize_or_zero() * delta * ANT_SPEED;
            match towards_target.x {
                x if x > 0. => { ant.moving_right = true; },
                x if x < 0. => { ant.moving_right = false; },
                _ => { },
            }
        }

        // And if this causes the current cell to change, move ant appropriately
        let new_cell_pos = pixel_to_hex(ant.pos);
        if let Some(i) = neighbors.iter().position(|(offset, _)| {
            cell_pos + *offset == new_cell_pos
        }) {
            if let Some(new_cell) = neighbors[i].1.as_mut() {
                ant.updated_this_frame = true;
                new_cell.ants.push(ant);
                continue;
            }
        }

        // Otherwise, add back to the normal cell
        cell.ants.push(ant);
    }
}

// Same considerations as above
fn draw_cell(cell: Ref<Cell>, x: i32, y: i32, ant_tex: &Texture2D, ant_grab_tex: &Texture2D, food_tex: &Texture2D) {
    let graphical_pos = hex_to_pixel(ivec2(x, y));
    let mut color = GREEN;
    color.b += cell.pheremone_searching as f32 / 10.0 - cell.pheremone_found as f32 / 5.0;
    color.r += cell.pheremone_found as f32 / 5.0 - cell.pheremone_searching as f32 / 10.0;
    draw_hexagon(
        graphical_pos.x,
        graphical_pos.y,
        HEX_RADIUS - HEX_RADIUS / 10.0 / 2.0,
        HEX_RADIUS / 10.0,
        true,
        DARKGREEN,
        color,
    );

    match cell.land {
        Land::Empty | Land::Nest(_) => {}
        Land::Food(_) => {
            draw_texture_ex(
                food_tex,
                graphical_pos.x - 75.0,
                graphical_pos.y - 75.0,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(150., 150.)),
                    ..Default::default()
                },
            );
        },
    }

    for ant in &cell.ants {
        draw_texture_ex(
            if ant.carrying_food {
                ant_grab_tex
            } else {
                ant_tex
            },
            ant.pos.x - 32.0, ant.pos.y - 32.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(64., 64.)),
                flip_x: ant.moving_right,
                ..Default::default()
            }
        );
    }

    match cell.land {
        Land::Empty => {}
        Land::Nest(food) => {
            draw_text(
                "Nest",
                graphical_pos.x - HEX_RADIUS / 1.7,
                graphical_pos.y - HEX_RADIUS / 4.0,
                HEX_RADIUS / 1.5,
                BLACK,
            );
            draw_text(
                food.to_string(),
                graphical_pos.x - HEX_RADIUS / 5.0,
                graphical_pos.y + HEX_RADIUS / 4.0,
                HEX_RADIUS / 1.5,
                BLACK,
            );
        }
        Land::Food(food) => {
            draw_text(
                food.to_string(),
                graphical_pos.x - HEX_RADIUS / 4.0,
                graphical_pos.y + HEX_RADIUS / 5.0,
                HEX_RADIUS / 1.5,
                BLACK,
            );
        },
    }
}

#[macroquad::main("Ant Simulation")]
async fn main() {
    let ant_tex = load_texture("art/ant.png").await.unwrap();
    let ant_grab_tex = load_texture("art/ant_grab.png").await.unwrap();
    let food_tex = load_texture("art/cabbage.png").await.unwrap();

    let mut cam = Camera2D {
        target: vec2(0., 0.0),
        ..Default::default()
    };

    let mut cam_zoom = 0.0005;

    let mut grid =
        HexGrid::<RefCell<Cell>>::from_bounds_hexagon(RefCell::new(Cell::default()), 0, 0, 6);
    grid.set(
        0,
        0,
        RefCell::new(Cell {
            land: Land::Nest(0),
            ants: iter::repeat_n(Ant { pos: vec2(0.0, 0.0), carrying_food: false, moving_right: false, updated_this_frame: false, target: None }, 30).collect(),
            ..Default::default()
        }),
    );

    let mut fps_queue = VecDeque::new();

    loop {
        // Deltatime shouldn't go to huge numbers for lag spikes, so cap it
        let delta = get_frame_time().min(0.1 /* min. 10 fps */);

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
            // Add food to tiles
            if is_mouse_button_pressed(MouseButton::Left) {
                let tile_pos = mouse_hex_position(&cam);
                if let Some(tile) = grid.get(tile_pos.x, tile_pos.y) {
                    let tile_land = tile.borrow().land.clone();
                    match tile_land {
                        Land::Empty => {
                            tile.borrow_mut().land = Land::Food(1);
                        },
                        Land::Nest(food) => {
                            tile.borrow_mut().land = Land::Nest(food + 1);
                        },
                        Land::Food(food) => {
                            tile.borrow_mut().land = Land::Food(food + 1);
                        },
                    }
                } else {
                    grid.set(tile_pos.x, tile_pos.y, RefCell::new(Cell {
                        land: Land::Empty,
                        ..Default::default()
                    }));
                }
            }
        }

        // UPDATE
        {
            // Nest spawns ants
            if let Some(mut nest) = grid.get(0, 0).map(RefCell::borrow_mut) && let Land::Nest(ref mut food) = nest.land {
                // Random frequency, avg 1 ant per 2 seconds
                if *food >= 4 && rand::gen_range(0, get_fps().max(1) * 2) == 0 {
                    *food -= 4;
                    nest.ants.push(Ant { pos: vec2(0., 0.), carrying_food: false, moving_right: false, updated_this_frame: false, target: None });
                }
            } else {
                unreachable!();
            }


        }

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
                                let mut it = above_row.0.iter().chain(above_row.1.iter());
                                let first = x - above_x_offset;
                                if first < -1 {
                                    (None, None)
                                } else if first == -1 {
                                    (None, it.next())
                                } else {
                                    let mut it = it.skip(first as usize);
                                    (it.next(), it.next())
                                }
                            })
                            .unwrap_or((None, None));

                        let left = prev;
                        let right = row_iter.peek().copied();

                        let (bottom_left, bottom_right) = below
                            .map(|(below_x_offset, below_row)| {
                                let mut it = below_row.0.iter().chain(below_row.1.iter());
                                let first = x - 1 - below_x_offset;
                                if first < -1 {
                                    (None, None)
                                } else if first == -1 {
                                    (None, it.next())
                                } else {
                                    let mut it = it.skip(first as usize);
                                    (it.next(), it.next())
                                }
                            })
                            .unwrap_or((None, None));

                        {
                            // All operations on a cell are performed in the same loop.
                            update_cell(
                                delta,
                                ivec2(x, y),
                                        cell.borrow_mut(),
                                        top_left.map(RefCell::borrow_mut),
                                        top_right.map(RefCell::borrow_mut),
                                        left.map(RefCell::borrow_mut),
                                        right.map(RefCell::borrow_mut),
                                        bottom_left.map(RefCell::borrow_mut),
                                        bottom_right.map(RefCell::borrow_mut),
                            );

                            draw_cell(cell.borrow(), x, y, &ant_tex, &ant_grab_tex, &food_tex);
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
            let hex_pos = mouse_hex_position(&cam);
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

        fps_queue.push_back(get_fps());
        if fps_queue.len() > 60 {
            fps_queue.pop_front();
        }
        {
            let top_left = cam.screen_to_world(vec2(0., 0.));
            let text_bottom = cam.screen_to_world(vec2(0., 32.));
            let font_size = text_bottom.y - top_left.y;
            draw_text(&format!("FPS: {}", fps_queue.iter().sum::<i32>()/60), top_left.x, top_left.y + font_size * 0.5, font_size, crate::WHITE);
        }
        next_frame().await
    }
}
