mod graphics;
mod quadtree;
mod raycast;

use my_math::prelude::*;
use minifb::MouseMode;

#[macro_use]
extern crate my_math;

use graphics::*;

use quadtree::*;
use raycast::*;

use std::time::Instant;
use std::env;

const BG_COL: u32 = (51 << 16) + (76 << 8) + 76;

static mut RES: i32 = 720;
static mut TREE_RES:i32 = unsafe { RES * 9/10 };
static mut HEIGHT: i32 = 1 << 5;
static mut CELL_SIZE: i32 = unsafe { TREE_RES/HEIGHT };

fn from_cell(pos:i32) -> i32 {
    unsafe { (pos * CELL_SIZE) + ((RES - HEIGHT * CELL_SIZE) / 2) }
}
fn from_cell_f32(pos:f32) -> i32 {
    unsafe { (pos * CELL_SIZE as f32).round() as i32 + ((RES - HEIGHT * CELL_SIZE) / 2) }
}
fn into_cell(pos:f32) -> f32 {
    unsafe { (pos - ((RES - HEIGHT * CELL_SIZE) / 2) as f32) / CELL_SIZE as f32 }
}

fn parse_args() -> bool {
    let args:Vec<String> = env::args().skip(1).collect();
    let mut full = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-f" => {
                full = true;
                i += 1;
            }
            "-r" => { unsafe {
                RES = args[i + 1].parse().unwrap_or_else(|_| {
                    panic!("|| resolution given is not a number ({})||",args[i + 1]); 
                });
                println!("res = {}",args[i + 1].parse::<i32>().unwrap());
                i += 2;
            }}
            "-d" => { unsafe {
                HEIGHT = 1 << args[i + 1].parse::<i32>().unwrap_or_else(|_| {
                    panic!("|| depth given is not a number ({})||",args[i + 1]); 
                });
                println!("depth = {}",args[i + 1]);
                i += 2;
            }}
            _ => panic!("not a valid flag \"{}\"",args[i]),
        }
    }

    unsafe {
        TREE_RES =  RES * 9/10 ;
        CELL_SIZE =  TREE_RES/HEIGHT ;
    }
    return full;
}


fn clear_screen() {
    use std::io::Write;
    print!("\x1b[2J\x1b[H");
    std::io::stdout().flush().unwrap();
}

fn main() {
    let full = parse_args();
    let mut quadtree;
    if full {
        quadtree = unsafe { Quadtree::new_full(HEIGHT,ivec2!(0,0)) };
    } else {
        quadtree = unsafe { Quadtree::new(HEIGHT,ivec2!(0,0)) };
    }

    let mut app = unsafe { App::new("raycast", RES, RES) };
    let mut target_x = unsafe { RES as f32/2. + 1e-5 };
    let mut target_y = unsafe { RES as f32/2. - 1e-5 };

    'draw_loop: while app.window.is_open() {
        let fb = &mut app.framebuffer;

        fb.clear(BG_COL);

        quadtree.draw_outline(fb);

        if let Some((mouse_x,mouse_y)) = app.window.get_mouse_pos(MouseMode::Discard) {
            let cell_mouse_x = into_cell(mouse_x);
            let cell_mouse_y = into_cell(mouse_y);
            let cell_target_x = into_cell(target_x);
            let cell_target_y = into_cell(target_y);

            let ray_origin = Vec2{ x: cell_mouse_x, y: cell_mouse_y};
            let ray_dir = Vec2{ x: cell_target_x - cell_mouse_x, 
                                y: cell_target_y - cell_mouse_y }.norm();

            let start = Instant::now();
            for _ in 0..1000 {
                dda_quad(ray_origin,ray_dir,1000.,&quadtree);
            }
            println!("dda   {:?}",start.elapsed());

            let start = Instant::now();
            for _ in 0..1000 {
                raycast2(ray_origin,ray_dir,&quadtree);
            }
            println!("param {:?}",start.elapsed());


            let collition = raycast2(ray_origin,ray_dir,&quadtree);
            
            if let Some((first_node,t)) = collition {
                unsafe {
                    fb.empty_square(from_cell(first_node.position.x) ,   from_cell(first_node.position.y) , first_node.size * CELL_SIZE , GREEN);
                    fb.empty_square(from_cell(first_node.position.x) + 1,from_cell(first_node.position.y) + 1, first_node.size * CELL_SIZE -2, GREEN);
                }
                let hit_pos = ray_origin + ray_dir * t;
                fb.circle(from_cell_f32(hit_pos.x),from_cell_f32(hit_pos.y),4,PINK);
            }

            let mouse_x = mouse_x.round() as i32;
            let mouse_y = mouse_y.round() as i32;
            fb.circle(mouse_x,mouse_y,20,RED);
            fb.circle(target_x.round() as i32 ,target_y.round() as i32,10,GREEN);
            fb.line(mouse_x,mouse_y,target_x.round() as i32,target_y.round() as i32,WHITE);
        }

        use minifb::MouseButton;
        if app.window.get_mouse_down(MouseButton::Left) {
            if let Some((mouse_x,mouse_y)) = app.window.get_mouse_pos(MouseMode::Discard) {
                let grid_x = into_cell(mouse_x).floor() as i32;
                let grid_y = into_cell(mouse_y).floor() as i32;
                let start = Instant::now();
                for _ in 0..1000 {
                    quadtree.add_block(ivec2!(grid_x,grid_y));
                }
                println!("add block: {:?}",start.elapsed());
            }
        }
        if app.window.get_mouse_down(MouseButton::Right) {
            if let Some((mouse_x,mouse_y)) = app.window.get_mouse_pos(MouseMode::Discard) {
                let grid_x = into_cell(mouse_x).floor() as i32;
                let grid_y = into_cell(mouse_y).floor() as i32;
                quadtree.remove_block(ivec2!(grid_x ,grid_y ));
            }
        }

        for key in app.window.get_keys() {
            use minifb::Key;
            match key {
                Key::Escape => break 'draw_loop,
                Key::Left => target_x  -= 5.,
                Key::Right => target_x += 5.,
                Key::Up => target_y    -= 5.,
                Key::Down => target_y  += 5.,
                _ => (),
            }
        }
        print!("len {} ",quadtree.nodes.len());
        for (i,node) in quadtree.nodes.iter().enumerate() {
            print!("{} ",if node.is_orphan { 1 } else { 0 });
            if i % 4 == 0 {
                print!("|");
            }
            if i % unsafe {HEIGHT} as usize == 0 {
                println!();
            }
        }
        println!();

        app.display();
        clear_screen();
    }
}


pub fn dda_quad(start: Vec2, dir: Vec2, max_distance: f32,chunk_data:&Quadtree) -> Option<(IVec2,Vec2)>{
    let mut voxel = IVec2::new(
                            start.x.floor() as i32, 
                            start.y.floor() as i32, 
                        );

    let step_dir = IVec2::new(
                            dir.x.signum() as i32 ,
                            dir.y.signum() as i32 ,
                        );

    let t_delta = Vec2::new( 
                            1. / dir.x.abs(), 
                            1. / dir.y.abs(), 
                        );

    fn frac0(x: f32) -> f32 {
        x - x.floor()
    }

    fn frac1(x: f32) -> f32 {
        1. - frac0(x)
    }

    let mut t_max_x = if dir.x > 0. {
        t_delta.x * frac1(start.x)
    } else {
        t_delta.x * frac0(start.x)
    };

    let mut t_max_y = if dir.y > 0. {
        t_delta.y * frac1(start.y)
    } else {
        t_delta.y * frac0(start.y)
    };

    let mut traveled_distance = 0.0;
    while traveled_distance < max_distance {

        if chunk_data.is_solid_at(voxel) {
            return Some((voxel,start + dir * traveled_distance));
        }

        if t_max_x < t_max_y {
            voxel.x += step_dir.x;
            traveled_distance = t_max_x;
            t_max_x += t_delta.x;
        } else {
            voxel.y += step_dir.y;
            traveled_distance = t_max_y;
            t_max_y += t_delta.y;
        }

    }
    None
}
