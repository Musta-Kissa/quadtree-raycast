#![allow(dead_code)]
use crate::quadtree_arr::*;
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

pub fn raycast2<'a>(start: Vec2, dir: Vec2, quadtree: &'a Quadtree) -> Option<(&'a QuadtreeNode,f32)> {
    let mut start = start;

    let root_node_idx = 0;
    let root_node = &quadtree.nodes[root_node_idx];

    let quad_size = root_node.size ;
    let quad_pos = root_node.position ;

    let mut mask:u8 = 0;
    if dir.x < 0. {
        start.x = 2. * quad_pos.x as f32 + root_node.size as f32 - start.x;
        mask |= 1;
    }
    if dir.y < 0. {
        start.y = 2. * quad_pos.y as f32 + root_node.size as f32 - start.y;
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

    return proc_subtree(start,dir,mask,quadtree,root_node_idx,tx0,ty0,tx1,ty1);

    fn proc_subtree(start: Vec2, dir: Vec2, mask: u8,quadtree: &Quadtree, node_idx: usize,tx0:f32,ty0:f32,tx1:f32,ty1:f32) -> Option<(&QuadtreeNode,f32)> {
        if !( tx1 >= 0. && ty1 >= 0. ) {
            return None;
        }

        let node = &quadtree.nodes[node_idx];
        if node.children_indexes.is_none() {
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

        let children_indexes = &node.children_indexes.as_ref().unwrap();

        while curr_node != 4 {
            let child_idx = children_indexes[curr_node as usize ^ mask as usize];
            match curr_node {
                0 => {
                    if let Some((hit,t)) = proc_subtree(start, dir, mask, quadtree,child_idx, tx0,ty0,txm,tym ) {
                        return Some((hit,t));
                    }
                    curr_node = next_node(curr_node,txm,tym);
                },
                1 => {
                    if let Some((hit,t)) = proc_subtree(start, dir, mask, quadtree,child_idx, txm,ty0,tx1,tym ) {
                        return Some((hit,t));
                    }
                    curr_node = next_node(curr_node,tx1,tym);
                },
                2 => {
                    if let Some((hit,t)) = proc_subtree(start, dir, mask, quadtree,child_idx, tx0,tym,txm,ty1) {
                        return Some((hit,t));
                    }
                    curr_node = next_node(curr_node,txm,ty1);
                },
                3 => {
                    if let Some((hit,t)) = proc_subtree(start, dir, mask, quadtree,child_idx, txm,tym,tx1,ty1) {
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
