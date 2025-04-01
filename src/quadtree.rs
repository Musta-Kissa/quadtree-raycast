use my_math::prelude::*;
use crate::graphics::Framebuffer;
use crate::CELL_SIZE;
use crate::from_cell;

pub struct QuadtreeNode {
    pub is_full: bool,

    pub children: Option<[Box<QuadtreeNode>;4]>,
    pub size: i32,
    pub position: IVec2,
}
impl QuadtreeNode {
    pub fn new(size: i32,pos: IVec2,full: bool) -> Self {
        QuadtreeNode {
            is_full: full,

            children: None,
            size: size,
            position: pos,
        }
    }
    pub fn is_leaf(&self) -> bool {
        self.children.is_none()
    }
    pub fn remove_block(&mut self, pos: IVec2) {
        if !self.is_full && self.children.is_none() {
            // isn't full and a leaf => is empty => nothing to do
            return;
        }
        if self.size == 1 {
            assert!(self.children.is_none());
            self.is_full = false;
            return;
        }

        if self.is_full && self.children.is_none() {
            // is full and a leaf => devide and call recursively on the proper node
            self.devide(true);

            let children = self.children.as_mut().unwrap();

            let rel_pos = pos - self.position;
            if rel_pos.x < self.size/2 && rel_pos.y < self.size/2 {
                children[0].remove_block(pos); 
            } else if rel_pos.x < self.size && rel_pos.y < self.size/2 {
                children[1].remove_block(pos); 
            } else if rel_pos.x < self.size/2 && rel_pos.y < self.size {
                children[2].remove_block(pos); 
            } else {
                children[3].remove_block(pos); 
            }
            self.is_full = false;
            return;
        }
        if let Some(children) = &mut self.children {
            // is mixed so not full and has children => just call recursively and check if removed
            // the last child if so merge the four nodes into an empty node

            let rel_pos = pos - self.position;
            if rel_pos.x < self.size/2 && rel_pos.y < self.size/2 {
                children[0].remove_block(pos); 
            } else if rel_pos.x < self.size && rel_pos.y < self.size/2 {
                children[1].remove_block(pos); 
            } else if rel_pos.x < self.size/2 && rel_pos.y < self.size {
                children[2].remove_block(pos); 
            } else {
                children[3].remove_block(pos); 
            }
            // if any child after removing is full return else merge
            for child in children {
                if child.is_full || child.children.is_some() {
                    return;
                }
            }
            // its already not full so just remove the children and return
            self.children = None;
            return;
        }

    }
    pub fn add_block(&mut self, pos: IVec2) {
        if self.is_full {
            return;
        }
        if self.size == 1 {
            self.is_full = true;
            return;
        } 
        if self.children.is_none() {
            // is a leaf and not full so its empty => devide and call recursively
            assert!(self.children.is_none());
            self.devide(false);

            let children = self.children.as_mut().unwrap();

            let rel_pos = pos - self.position;
            if rel_pos.x < self.size/2 && rel_pos.y < self.size/2 {
                children[0].add_block(pos); 
            } else if rel_pos.x < self.size && rel_pos.y < self.size/2 {
                children[1].add_block(pos); 
            } else if rel_pos.x < self.size/2 && rel_pos.y < self.size {
                children[2].add_block(pos); 
            } else {
                children[3].add_block(pos); 
            }
            // is empty so one addition cant make it full
            return;
        } else {
            // isn't a leaf and isn't full => dont devide call recursively and check if filled if
            // so merge
            let children = self.children.as_mut().unwrap();

            let rel_pos = pos - self.position;
            if rel_pos.x < self.size/2 && rel_pos.y < self.size/2 {
                children[0].add_block(pos); 
            } else if rel_pos.x < self.size && rel_pos.y < self.size/2 {
                children[1].add_block(pos); 
            } else if rel_pos.x < self.size/2 && rel_pos.y < self.size {
                children[2].add_block(pos); 
            } else {
                children[3].add_block(pos); 
            }

            // check if full 
            for child in children {
                if !child.is_full {
                    return;
                }
            }
            self.is_full = true;
            self.children = None;
        }
    }
    pub fn draw_outline(&self, fb: &mut Framebuffer) {

        if let Some(children) = &self.children {
            for child in children {
                child.draw_outline(fb);
            }
        } else if self.is_full {
            unsafe {fb.square(from_cell(self.position.x) ,from_cell(self.position.y), self.size * CELL_SIZE , 0)};
        }
        unsafe {fb.empty_square(from_cell(self.position.x) ,from_cell(self.position.y) , self.size * CELL_SIZE , !0)};
    }
    pub fn devide(&mut self,full: bool) {
        //self.is_leaf = false;

        let half_size = self.size /2;
        let pos = self.position;
        // + ----- + ----- +
        // |  q2   |  q3   |
        // |       |       |
        // + ----- + ----- +
        // |  q0   |  q1   |
        // |       |       |
        // + ----- + ----- +
        self.children = Some ([ 
                // q0
                Box::new(QuadtreeNode::new(half_size,
                                    pos, full)),                                       
                // q1
                Box::new(QuadtreeNode::new(half_size,
                                    ivec2!(pos.x + half_size , pos.y ),full)),        
                // q2
                Box::new(QuadtreeNode::new(half_size,
                                    ivec2!(pos.x , pos.y + half_size ),full)),         
                // q3
                Box::new(QuadtreeNode::new(half_size,
                                    ivec2!(pos.x + half_size, pos.y + half_size),full)),  
            ])
    }
}
pub struct Quadtree {
    pub head: QuadtreeNode
}
impl Quadtree {
    pub fn new(size: i32, pos: IVec2) -> Self {
        let mut s = size ;
        while s != 1 {
            assert!(s % 2 == 0, "the size of the quad tree must be a power of two");
            s /= 2;
        }
        Quadtree {
            head: QuadtreeNode::new(size,pos,false),
        }
    }
    pub fn new_full(size: i32, pos: IVec2) -> Self {
        let mut s = size ;
        while s != 1 {
            assert!(s % 2 == 0, "the size of the quad tree must be a power of two");
            s /= 2;
        }
        Quadtree {
            head: QuadtreeNode::new(size,pos,true),
        }
    }
    pub fn add_block(&mut self,pos: IVec2) {
        if pos.x < self.head.position.x || pos.x >= self.head.position.x + self.head.size ||
            pos.y < self.head.position.y || pos.y >= self.head.position.y + self.head.size {
            return;
        }
        self.head.add_block(pos);
    }
    pub fn remove_block(&mut self,pos: IVec2) {
        if pos.x < self.head.position.x || pos.x >= self.head.position.x + self.head.size ||
            pos.y < self.head.position.y || pos.y >= self.head.position.y + self.head.size {
            return;
        }
        self.head.remove_block(pos);
    }
    pub fn is_solid_at(&self,pos: IVec2) -> bool {
        if pos.x < self.head.position.x || pos.x >= self.head.position.x + self.head.size ||
            pos.y < self.head.position.y || pos.y >= self.head.position.y + self.head.size {
            return false;
        }
        let mut curr = &self.head;
        loop {
            if curr.is_full {
                return true;
            }
            if curr.children.is_none() || curr.size == 1 {
                return false;
            }

            let children = curr.children.as_ref().unwrap();
            let rel_pos = pos - curr.position;
            if rel_pos.x < curr.size/2 && rel_pos.y < curr.size/2 {
                curr = &children[0];
            } else if rel_pos.x < curr.size && rel_pos.y < curr.size/2 {
                curr = &children[1];
            } else if rel_pos.x < curr.size/2 && rel_pos.y < curr.size {
                curr = &children[2];
            } else {
                curr = &children[3];
            }
        }
    }
    pub fn index_at(&self, pos: IVec2) -> i32 {
        if pos.x < self.head.position.x || pos.x >= self.head.position.x + self.head.size ||
            pos.y < self.head.position.y || pos.y >= self.head.position.y + self.head.size {
            return 1;
        }
        let mut curr = &self.head;
        let mut curr_idx = -1;
        loop {
            if curr.is_full || curr.children.is_none() {
                return curr_idx;
            }
            if curr.size == 1 {
                return curr_idx;
            }

            let children = curr.children.as_ref().unwrap();
            let rel_pos = pos - curr.position;
            if rel_pos.x < curr.size/2 && rel_pos.y < curr.size/2 {
                curr = &children[0];
                curr_idx = 0;
            } else if rel_pos.x < curr.size && rel_pos.y < curr.size/2 {
                curr = &children[1];
                curr_idx = 1;
            } else if rel_pos.x < curr.size/2 && rel_pos.y < curr.size {
                curr = &children[2];
                curr_idx = 2;
            } else {
                curr = &children[3];
                curr_idx = 3;
            }
        }
    }
    pub fn size_at(&self, pos: IVec2) -> i32 {
        if pos.x < self.head.position.x || pos.x >= self.head.position.x + self.head.size ||
            pos.y < self.head.position.y || pos.y >= self.head.position.y + self.head.size {
            return 1;
        }
        let mut curr = &self.head;
        loop {
            if curr.is_full || curr.children.is_none() {
                return curr.size;
            }
            if curr.size == 1 {
                return 1;
            }

            let children = curr.children.as_ref().unwrap();
            let rel_pos = pos - curr.position;
            if rel_pos.x < curr.size/2 && rel_pos.y < curr.size/2 {
                curr = &children[0];
            } else if rel_pos.x < curr.size && rel_pos.y < curr.size/2 {
                curr = &children[1];
            } else if rel_pos.x < curr.size/2 && rel_pos.y < curr.size {
                curr = &children[2];
            } else {
                curr = &children[3];
            }
        }
    }
    pub fn draw_outline(&self, fb: &mut Framebuffer) {
        self.head.draw_outline(fb);
    }
}
