#![allow(dead_code)]

pub const BLACK:  u32 = 0;
pub const WHITE:  u32 = !0;
pub const RED:    u32 = ((1 << 8) - 1) << 16;
pub const GREEN:  u32 = ((1 << 8) - 1) << 8;
pub const BLUE:   u32 = ((1 << 8) - 1) << 0;
pub const PINK: u32 = RED | BLUE;
pub const YELLOW: u32 = GREEN | BLUE;

// The order is reversed in memory
#[derive(Clone, Copy)]
pub struct ColorChanels {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}
#[derive(Clone, Copy)]
pub union Color {
    pub col: u32,
    pub ch: ColorChanels,
}
pub fn blend_color(c1: Color, c2: Color, ratio: f32) -> Color {
    unsafe {
        Color {
            ch: ColorChanels {
                a: c1.ch.a,
                r: (c1.ch.r as f32 * (1. - ratio) + c2.ch.r as f32 * ratio).round() as u8,
                g: (c1.ch.g as f32 * (1. - ratio) + c2.ch.g as f32 * ratio).round() as u8,
                b: (c1.ch.b as f32 * (1. - ratio) + c2.ch.b as f32 * ratio).round() as u8,
            },
        }
    }
}

pub struct App {
    pub window: minifb::Window,
    pub framebuffer: Framebuffer,
}
impl App {
    pub fn new(name: &str, width: i32, height: i32) -> Self {
        let mut window =
            minifb::Window::new(name, width as usize, height as usize, minifb::WindowOptions { resize: true , ..minifb::WindowOptions::default() } ).unwrap();

        window.set_target_fps(60);

        let framebuffer = Framebuffer::new(width as usize, height as usize);
        Self {
            window,
            framebuffer,
        }
    }
    pub fn display(&mut self) {
        let _ = self.window.update_with_buffer(
            &self.framebuffer.data,
            self.framebuffer.width,
            self.framebuffer.height,
        );
    }
}
pub struct Framebuffer {
    pub data: Vec<u32>,
    pub width: usize,
    pub height: usize,
}
impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let data = vec![0; width * height];
        Self {
            data,
            width,
            height,
        }
    }
    pub fn clear(&mut self, color: u32) {
        let surface_len = self.height * self.width;
        for i in 0..surface_len {
            self.data[i] = color;
        }
    }
    pub fn pixel_fits(&self, pos_x:i32,pos_y:i32) -> bool {
        !(pos_y < 0
            || pos_x < 0
            || pos_y >= self.height as i32
            || pos_x >= self.width  as i32)
    }

    pub fn set_pixel(&mut self, pos_x:i32,pos_y:i32, color: u32) {
        if !self.pixel_fits(pos_x,pos_y) {
            return;
        }
        self.data[pos_y as usize * self.width + pos_x as usize] = color;
    }
    pub fn square(&mut self, pos_x: i32, pos_y: i32,size: i32,color:u32) {
        for x in pos_x ..= pos_x+size {
            for y in pos_y ..= pos_y+size {
                self.set_pixel(x,y,color);
            }
        }
    }
    pub fn empty_square(&mut self, pos_x: i32, pos_y: i32,size: i32,color:u32) {
        self.line(pos_x, pos_y, pos_x + size,pos_y , color);

        self.line(pos_x+ size,pos_y,pos_x+size,pos_y + size, color);

        self.line(pos_x+ size,pos_y+size,pos_x ,pos_y + size, color);
        self.line(pos_x,pos_y,pos_x ,pos_y + size + 1, color);
    }
    pub fn line(&mut self, start_x:i32,start_y:i32,end_x:i32,end_y:i32,color:u32) {
        let d_y: i32 = (end_y - start_y).abs();
        let d_x: i32 = (end_x - start_x).abs();

        let s_x: i32 = if start_x < end_x { 1 } else { -1 };
        let s_y: i32 = if start_y < end_y { 1 } else { -1 };

        let mut curr_y = start_y;
        let mut curr_x = start_x;

        let mut err = d_x - d_y;

        while !(curr_y == end_y && curr_x == end_x) {
            self.set_pixel(curr_x, curr_y, color);

            let e2 = err * 2;
            if e2 > -d_y {
                err -= d_y;
                curr_x += s_x;
            }
            if e2 < d_x {
                err += d_x;
                curr_y += s_y;
            }
        }
    }

    // Function for circle-generation
    // using Bresenham's algorithm
    pub fn circle(&mut self,xc: i32, yc: i32, r: i32,color:u32){
        #[allow(non_snake_case)]
        let mut drawCircle = |xc,yc,x,y| {
            self.set_pixel(xc+x, yc+y, color);
            self.set_pixel(xc-x, yc+y, color);
            self.set_pixel(xc+x, yc-y, color);
            self.set_pixel(xc-x, yc-y, color);
            self.set_pixel(xc+y, yc+x, color);
            self.set_pixel(xc-y, yc+x, color);
            self.set_pixel(xc+y, yc-x, color);
            self.set_pixel(xc-y, yc-x, color);
        };
        let mut x = 0;
        let mut y = r;
        let mut d = 3 - 2 * r;
        drawCircle(xc, yc, x, y);
        while y >= x {
            if d > 0 {
                y -= 1; 
                d = d + 4 * (x - y) + 10;
            } else {
                d = d + 4 * x + 6;
            }
            x += 1;
            drawCircle(xc, yc, x, y);
        }
    }
}

