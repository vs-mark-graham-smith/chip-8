// use sdl2::rect::Rect;
use sdl2::render::WindowCanvas;
use sdl2::rect::Rect;
use sdl2::pixels::Color;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const SCALE: isize = 20;

pub struct Display {
    gfx: [[u8; 64]; 32],
    draw_flag: bool,
    pub sdl: sdl2::Sdl,
    pub canvas: WindowCanvas
}

impl Display {
    pub fn new() -> Display {
        let sdl = sdl2::init().unwrap();
        let canvas = sdl.video()
            .unwrap()
            .window(
                "Chip-8",
                (WIDTH as isize * SCALE) as u32,
                (HEIGHT as isize * SCALE) as u32,
            )
            .position_centered()
            .build()
            .unwrap()
            .into_canvas()
            .build()
            .unwrap();

        Display {
            gfx: [[0; WIDTH]; HEIGHT],
            draw_flag: true,
            sdl,
            canvas
        }
    }

    pub fn clear(&mut self) {
        println!("Cleared");
        self.gfx = [[0; WIDTH]; HEIGHT];
        self.draw_flag = true;
    }

    pub fn draw(&mut self, x: usize, y: usize, sprite: &[u8]) -> u8 {
        let mut collision = 0u8;
        let n = sprite.len() as usize;
        let mut yj: usize;
        let mut xi: usize;

        for j in 0..n {
            for i in 0..8 {
                yj = (y + j) % HEIGHT;
                xi = (x + i) % WIDTH;

                if (sprite[j] & (0x80 >> i)) != 0 {
                    if self.gfx[yj][xi] == 1 { collision = 1 }
                    self.gfx[yj][xi] ^= 1;
                }
            }
        }

        self.draw_flag = true;
        collision
    }

    pub fn draw_screen(&mut self) {
        println!("GFX: ");
        for row in self.gfx {
            println!("{:?}", row);
        }

        if !self.draw_flag { return }

        let mut pixel: u8;
        let sc = SCALE as u16;
        let pt = |p: usize| { (p as i16) * (SCALE as i16) };

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                pixel = if self.gfx[y][x] != 0 { 255 } else { 0 };
                self.canvas.set_draw_color(Color::RGB(pixel,pixel,pixel));
                self.canvas.fill_rect(
                    Some(Rect::new(
                        pt(x) as i32,
                        pt(y) as i32,
                        sc as u32,
                        sc as u32
                    )),
                ).unwrap();
            }
        }


        self.canvas.present();
        self.draw_flag = false;
    }
}
