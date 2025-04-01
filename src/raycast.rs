#![allow(dead_code)]
use crate::quadtree::*;
use my_math::prelude::*;

    // + ----- + ----- +
    // |  q2   |  q3   |
    // |       |       |
    // + ----- + ----- +
    // |  q0   |  q1   |
    // |       |       |
    // + ----- + ----- +


fn next_node(curr_quad: i32, tmx: f32, tmy: f32) -> i32 {
    const EXIT: i32 = 4;
    let exit_lookup = [
        [ 1,    2],
        [EXIT,  3],
        [ 3,   EXIT],
        [EXIT, EXIT]
    ];
    return exit_lookup[curr_quad as usize][if tmx <= tmy { 0 } else { 1 }];
}

fn first_node (tx0: f32, ty0: f32, tmx: f32, tmy: f32) -> i32 {
    // x entry plane
    if tx0 > ty0 {
        if tx0 < tmy {
            return 0;
        } else {
            return 2;
        }
    // y entry plane
    } else {
        if ty0 < tmx {
            return 0;
        } else {
            return 1;
        }
    }
}

pub fn raycast<'a>(start: Vec2, dir: Vec2, chunk_data: &'a Quadtree) -> Vec<&'a QuadtreeNode> {
    //let og_start = start;
    let mut start = start;

    let node = &chunk_data.head;

    let quad_size = node.size ;
    let quad_pos = node.position ;

    let mut mask:u8 = 0;
    if dir.x < 0. {
        start.x = 2. * quad_pos.x as f32 + node.size as f32 - start.x;
        mask |= 1;
    }
    if dir.y < 0. {
        start.y = 2. * quad_pos.y as f32 + node.size as f32 - start.y;
        mask |= 2;
    }

    let tx0 = (quad_pos.x as f32 - start.x) / dir.x.abs();
    let ty0 = (quad_pos.y as f32 - start.y) / dir.y.abs();

    let tx1 = (quad_pos.x as f32 + quad_size as f32 - start.x) / dir.x.abs();
    let ty1 = (quad_pos.y as f32 + quad_size as f32 - start.y) / dir.y.abs();

    let t_min = tx0.max(ty0);
    let t_max = tx1.min(ty1);

    let intersects: bool = t_min < t_max ;

    if !intersects {
        println!("no intersection");
        return Vec::new();
    }

    return proc_subtree(start,dir,mask,node,tx0,ty0,tx1,ty1);

    fn proc_subtree(start: Vec2, dir: Vec2, mask: u8,node: &QuadtreeNode,tx0:f32,ty0:f32,tx1:f32,ty1:f32) -> Vec<&QuadtreeNode> {
        if node.children.is_none() {
            return vec![node];
            //if !node.is_full {
                //return vec![node];
            //} else {
                //return Vec::new();
            //}
        }

        let txm = (tx0 + tx1) /2.;
        let tym = (ty0 + ty1) /2.;

        let mut curr_node = first_node(tx0,ty0,txm,tym);

        let children = &node.children.as_ref().unwrap();

        let mut out: Vec<&QuadtreeNode> = Vec::new();
        while curr_node != 4 {
            let child = &children[curr_node as usize ^ mask as usize];
            match curr_node {
                0 => {
                    out.extend_from_slice(
                            &proc_subtree(start,dir,mask,child, tx0,ty0,txm,tym ));
                    curr_node = next_node(curr_node,txm,tym);
                },
                1 => {
                    out.extend_from_slice(
                            &proc_subtree(start,dir,mask,child, txm,ty0,tx1,tym ));
                    curr_node = next_node(curr_node,tx1,tym);
                },
                2 => {
                    out.extend_from_slice(
                            &proc_subtree(start,dir,mask,child, tx0,tym,txm,ty1));
                    curr_node = next_node(curr_node,txm,ty1);
                },
                3 => {
                    out.extend_from_slice(
                            &proc_subtree(start,dir,mask,child, txm,tym,tx1,ty1));
                    curr_node = 4;
                },
                4 => (),
                _ => panic!(),
            }
        }
        return out;
    }
}

pub fn raycast2<'a>(start: Vec2, dir: Vec2, chunk_data: &'a Quadtree) -> Option<(&'a QuadtreeNode,f32)> {
    let mut start = start;

    let node = &chunk_data.head;

    let quad_size = node.size ;
    let quad_pos = node.position ;

    let mut mask:u8 = 0;
    if dir.x < 0. {
        start.x = 2. * quad_pos.x as f32 + node.size as f32 - start.x;
        mask |= 1;
    }
    if dir.y < 0. {
        start.y = 2. * quad_pos.y as f32 + node.size as f32 - start.y;
        mask |= 2;
    }

    let tx0 = (quad_pos.x as f32 - start.x) / dir.x.abs();
    let ty0 = (quad_pos.y as f32 - start.y) / dir.y.abs();

    let tx1 = (quad_pos.x as f32 + quad_size as f32 - start.x) / dir.x.abs();
    let ty1 = (quad_pos.y as f32 + quad_size as f32 - start.y) / dir.y.abs();

    let t_min = tx0.max(ty0);
    let t_max = tx1.min(ty1);

    let intersects: bool = t_min < t_max ;

    if !intersects {
        //println!("no intersection");
        return None;
    }

    return proc_subtree(start,dir,mask,node,tx0,ty0,tx1,ty1);

    fn proc_subtree(start: Vec2, dir: Vec2, mask: u8,node: &QuadtreeNode,tx0:f32,ty0:f32,tx1:f32,ty1:f32) -> Option<(&QuadtreeNode,f32)> {
        if node.children.is_none() {
            if node.is_full && ( tx1 >= 0. && ty1 >= 0. ) {
                if tx0 < 0. && ty0 < 0. {
                    return Some((node,0.));
                }else {
                    let t_min = tx0.max(ty0);
                    return Some((node,t_min));
                }
            } else {
                return None;
            }
        }

        let txm = (tx0 + tx1) /2.;
        let tym = (ty0 + ty1) /2.;

        let mut curr_node = first_node(tx0,ty0,txm,tym);

        let children = &node.children.as_ref().unwrap();

        while curr_node != 4 {
            let child = &children[curr_node as usize ^ mask as usize];
            match curr_node {
                0 => {
                    if let Some((hit,t)) = proc_subtree(start, dir, mask, child, tx0,ty0,txm,tym ) {
                        return Some((hit,t));
                    }
                    curr_node = next_node(curr_node,txm,tym);
                },
                1 => {
                    if let Some((hit,t)) = proc_subtree(start, dir, mask, child, txm,ty0,tx1,tym ) {
                        return Some((hit,t));
                    }
                    curr_node = next_node(curr_node,tx1,tym);
                },
                2 => {
                    if let Some((hit,t)) = proc_subtree(start, dir, mask, child, tx0,tym,txm,ty1) {
                        return Some((hit,t));
                    }
                    curr_node = next_node(curr_node,txm,ty1);
                },
                3 => {
                    if let Some((hit,t)) = proc_subtree(start, dir, mask, child, txm,tym,tx1,ty1) {
                        return Some((hit,t));
                    }
                    curr_node = 4;
                },
                4 => (),
                _ => panic!(),
            }
        }
        return None;
    }
}
