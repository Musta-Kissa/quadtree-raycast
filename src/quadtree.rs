use my_math::prelude::*;
use crate::graphics::Framebuffer;

use crate::from_cell;
use crate::CELL_SIZE;

#[derive(Copy,Clone)]
pub struct QuadtreeNode {
    pub is_full: bool,
    pub is_orphan: bool,

    pub children_indexes: Option<[usize;4]>,
    pub size: i32,
    pub position: IVec2,
}
impl QuadtreeNode {
    pub fn new(size: i32,pos: IVec2,full: bool) -> Self {
        QuadtreeNode {
            is_full: full,
            is_orphan: false,

            children_indexes: None,
            size: size,
            position: pos,
        }
    }
}

const ROOT_IDX:usize = 0;
pub struct Quadtree {
    pub nodes: Vec<QuadtreeNode>,
}
impl Quadtree {

    pub fn new(size: i32, pos: IVec2) -> Self {
        let mut s = size ;
        while s != 1 {
            assert!(s % 2 == 0, "the size of the quad tree must be a power of two");
            s /= 2;
        }
        Quadtree {
            nodes: vec![QuadtreeNode::new(size,pos,false)],
        }
    }
    pub fn new_full(size: i32, pos: IVec2) -> Self {
        let mut s = size ;
        while s != 1 {
            assert!(s % 2 == 0, "the size of the quad tree must be a power of two");
            s /= 2;
        }
        Quadtree {
            nodes: vec![QuadtreeNode::new(size,pos,true)],
        }
    }
    fn devide_node(&mut self,node_idx: usize,full:bool) {
        let len = self.nodes.len();
        let node = &self.nodes[node_idx];

        let half_size = node.size /2;
        let pos = node.position;
        let nodes = [
        // q0
            QuadtreeNode::new(half_size,pos, full),
        // q1
            QuadtreeNode::new(half_size,ivec2!(pos.x + half_size , pos.y ),full),
        // q2
            QuadtreeNode::new(half_size,ivec2!(pos.x , pos.y + half_size ),full),
        // q3
            QuadtreeNode::new(half_size,ivec2!(pos.x + half_size, pos.y + half_size),full),
        ];
        let mut idx_arr = [0;4];
        let mut i = 1;
        while i < len {
            if self.nodes[i].is_orphan {
                for j in 0 .. 4 {
                    assert!(self.nodes[i + j].is_orphan);
                    self.nodes[i + j] = nodes[j];
                    idx_arr[j] = i + j;
                }
                self.nodes[node_idx].children_indexes = Some(idx_arr);
                return;
            }
            i += 4;
        }
        self.nodes.extend_from_slice(&nodes);
        self.nodes[node_idx].children_indexes = Some([len,len+1,len+2,len+3]);
        return;
    }
    pub fn add_block(&mut self,pos: IVec2) {
        let head = &self.nodes[ROOT_IDX];
        if pos.x < head.position.x || pos.x >= head.position.x + head.size ||
            pos.y < head.position.y || pos.y >= head.position.y + head.size {
            return;
        }

        unsafe { add_block_recursion(self,pos,ROOT_IDX) };

        unsafe fn add_block_recursion(tree: &mut Quadtree,pos: IVec2,node_idx:usize) {
            // so rust wont complain, makes the code cleaner no need to index into tree.nodes every time
            let curr_node: &mut QuadtreeNode = &mut *((&mut tree.nodes[node_idx]) as *mut _);

            if curr_node.is_full {
                return;
            }
            if curr_node.size == 1 {
                curr_node.is_full = true;
                return;
            } 

            if curr_node.children_indexes.is_none() {
                // is a leaf and not full so its empty => devide and call recursively
                tree.devide_node(node_idx,false); 

                // the data might have moved in memory so update the reference 
                let curr_node: &mut QuadtreeNode = &mut *((&mut tree.nodes[node_idx]) as *mut _);

                let next_node_idx = pos_to_idx(curr_node,pos);

                // is empty so one addition cant make it full
                add_block_recursion(tree,pos,next_node_idx);

            } else if let Some(children_idx) = curr_node.children_indexes {
                // isn't a leaf and isn't full => dont devide call recursively and check if filled if
                // so merge
                
                let next_node_idx = pos_to_idx(curr_node,pos);
                add_block_recursion(tree,pos,next_node_idx);

                // the data might have moved in memory so update the reference 
                let curr_node: &mut QuadtreeNode = &mut *((&mut tree.nodes[node_idx]) as *mut _);

                // check if full 
                for child_idx in children_idx {
                    if !tree.nodes[child_idx].is_full {
                        return;
                    }
                }
                for child_idx in children_idx {
                    tree.nodes[child_idx].is_orphan = true;
                }

                curr_node.is_full = true;
                curr_node.children_indexes = None;
            }
        }
    }
    pub fn remove_block(&mut self,pos: IVec2) {
        let head = &self.nodes[ROOT_IDX];
        if pos.x < head.position.x || pos.x >= head.position.x + head.size ||
            pos.y < head.position.y || pos.y >= head.position.y + head.size {
            return;
        }
        unsafe { remove_block_recursion(self,pos,ROOT_IDX); }

        unsafe fn remove_block_recursion(tree: &mut Quadtree, pos: IVec2, node_idx: usize) {
            // so rust wont complain, makes the code cleaner no need to index into tree.nodes every time
            let curr_node: &mut QuadtreeNode = &mut *((&mut tree.nodes[node_idx]) as *mut _);

            if curr_node.size == 1 {
                assert!(curr_node.children_indexes.is_none());
                curr_node.is_full = false;
                return;
            }

            if curr_node.children_indexes.is_none() {
                if !curr_node.is_full {
                    // isn't full and a leaf => is empty => nothing to do
                    return;
                }
                // is full and a leaf => devide and call recursively on the proper node
                tree.devide_node(node_idx,true);
                // the data might have moved in memory so update the reference 
                let curr_node: &mut QuadtreeNode = &mut *((&mut tree.nodes[node_idx]) as *mut _);

                let next_node_idx = pos_to_idx(curr_node,pos);

                curr_node.is_full = false;
                remove_block_recursion(tree,pos,next_node_idx);

            } else if let Some(children_idx) = curr_node.children_indexes {
                // is mixed so not full and has children => just call recursively and check if removed
                // the last child if so merge the four nodes into an empty node

                let next_node_idx = pos_to_idx(curr_node,pos);

                remove_block_recursion(tree,pos,next_node_idx);
                // the data might have moved in memory so update the reference 
                let curr_node: &mut QuadtreeNode = &mut *((&mut tree.nodes[node_idx]) as *mut _);

                // if any child after removing is full return else merge
                for child_idx in children_idx {
                    let child = &tree.nodes[child_idx];
                    if child.is_full || child.children_indexes.is_some() {
                        return;
                    }
                }
                // its already not full so just remove the children and return
                for child_idx in children_idx {
                    tree.nodes[child_idx].is_orphan = true;
                }
                curr_node.children_indexes = None;
            }
        }
    }
    pub fn is_solid_at(&self,pos: IVec2) -> bool {
        let head = &self.nodes[0];
        if pos.x < head.position.x || pos.x >= head.position.x + head.size ||
            pos.y < head.position.y || pos.y >= head.position.y + head.size {
            return false;
        }
        let mut curr = head;
        loop {
            if curr.is_full {
                return true;
            }
            if curr.children_indexes.is_none() || curr.size == 1 {
                return false;
            }

            let children = curr.children_indexes.as_ref().unwrap();
            let rel_pos = pos - curr.position;
            if rel_pos.x < curr.size/2 && rel_pos.y < curr.size/2 {
                curr = &self.nodes[children[0]];
            } else if rel_pos.x < curr.size && rel_pos.y < curr.size/2 {
                curr = &self.nodes[children[1]];
            } else if rel_pos.x < curr.size/2 && rel_pos.y < curr.size {
                curr = &self.nodes[children[2]];
            } else {
                curr = &self.nodes[children[3]];
            }
        }
    }
    pub fn draw_outline(&self, fb: &mut Framebuffer) {
        draw_outline_recursion(self,fb,ROOT_IDX);

        fn draw_outline_recursion(tree:&Quadtree,fb: &mut Framebuffer,node_idx: usize) {
            let node = &tree.nodes[node_idx];
            if let Some(children_idx) = &node.children_indexes {
                for child_idx in children_idx {
                    draw_outline_recursion(tree,fb,*child_idx);
                }
            } else if node.is_full {
                unsafe {fb.square(from_cell(node.position.x) ,from_cell(node.position.y), node.size * CELL_SIZE , 0)};
            }
            unsafe {fb.empty_square(from_cell(node.position.x) ,from_cell(node.position.y) , node.size * CELL_SIZE , !0)};
        }
    }
}

fn pos_to_idx(node: &QuadtreeNode, pos: IVec2 ) -> usize {
        let children = node.children_indexes.as_ref().unwrap();

        let rel_pos = pos - node.position;
        if rel_pos.x < node.size/2 && rel_pos.y < node.size/2 {
            return children[0];
        } else if rel_pos.x < node.size && rel_pos.y < node.size/2 {
            return children[1];
        } else if rel_pos.x < node.size/2 && rel_pos.y < node.size {
            return children[2];
        } else {
            return children[3];
        }
}
