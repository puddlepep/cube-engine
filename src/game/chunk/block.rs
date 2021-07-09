
use cgmath::Vector3;
use image::{DynamicImage, GenericImageView};
use crate::game::color::Color;
use std::fs::{self, FileType};
use super::super::renderer;
use renderer::texture;

// pub static BLOCKS: &'static [Block; 4] = &[
//     Block { id: 0, name: "air", color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 } },
//     Block { id: 1, name: "grass", color: Color { r: 0.0, g: 1.0, b: 0.5, a: 1.0 } },
//     Block { id: 2, name: "dirt", color: Color { r: 0.5, g: 0.4, b: 0.3, a: 1.0 } },
//     Block { id: 3, name: "stone", color: Color { r: 0.4, g: 0.4, b: 0.4, a: 1.0 } }
// ];

pub struct Block {
    pub id: u8,
    pub name: String,
    pub sided: bool,
}

pub enum Side {
    Front,
    Back,
    Left,
    Right,
    Top,
    Bottom,
}

impl Block {
    pub const UP: Vector3<f32> = Vector3::new(0.0, 1.0, 0.0);
    pub const DOWN: Vector3<f32> = Vector3::new(0.0, -1.0, 0.0);
    pub const LEFT: Vector3<f32> = Vector3::new(-1.0, 0.0, 0.0);
    pub const RIGHT: Vector3<f32> = Vector3::new(1.0, 0.0, 0.0);
    pub const FORWARD: Vector3<f32> = Vector3::new(0.0, 0.0, -1.0);
    pub const BACKWARD: Vector3<f32> = Vector3::new(0.0, 0.0, 1.0);

    // Ordered CCW, from top right.
    pub fn get_tex_coords(&self, side: Side, atlas: &DynamicImage) -> ([f32; 2], [f32; 2], [f32; 2], [f32; 2]) {
        
        let tl: [f32; 2];
        let tr: [f32; 2];
        let bl: [f32; 2];
        let br: [f32; 2];

        let (atlas_x, atlas_y) = atlas.dimensions();
        let y_increment = 16.0 / atlas_y as f32;
        let x_increment = 16.0 / atlas_x as f32;

        if self.sided {
            match side {
                Side::Top => {
                    tl = [x_increment, y_increment * self.id as f32];
                    tr = [x_increment + x_increment, y_increment * self.id as f32];
                    bl = [x_increment, y_increment * self.id as f32 + y_increment];
                    br = [x_increment + x_increment, y_increment * self.id as f32 + y_increment];
                },
                Side::Bottom => {
                    tl = [x_increment + x_increment + x_increment, y_increment * self.id as f32];
                    tr = [x_increment + x_increment + x_increment + x_increment, y_increment * self.id as f32];
                    bl = [x_increment + x_increment + x_increment, y_increment * self.id as f32 + y_increment];
                    br = [x_increment + x_increment + x_increment + x_increment, y_increment * self.id as f32 + y_increment];
                },
                Side::Front => {
                    tl = [x_increment + x_increment, y_increment * self.id as f32];
                    tr = [x_increment + x_increment + x_increment, y_increment * self.id as f32];
                    bl = [x_increment + x_increment, y_increment * self.id as f32 + y_increment];
                    br = [x_increment + x_increment + x_increment, y_increment * self.id as f32 + y_increment];
                },
                Side::Back => {
                    tl = [x_increment + x_increment, y_increment * self.id as f32];
                    tr = [x_increment + x_increment + x_increment, y_increment * self.id as f32];
                    bl = [x_increment + x_increment, y_increment * self.id as f32 + y_increment];
                    br = [x_increment + x_increment + x_increment, y_increment * self.id as f32 + y_increment];
                },
                Side::Left => {
                    tl = [x_increment + x_increment, y_increment * self.id as f32];
                    tr = [x_increment + x_increment + x_increment, y_increment * self.id as f32];
                    bl = [x_increment + x_increment, y_increment * self.id as f32 + y_increment];
                    br = [x_increment + x_increment + x_increment, y_increment * self.id as f32 + y_increment];
                },
                Side::Right => {
                    tl = [x_increment + x_increment, y_increment * self.id as f32];
                    tr = [x_increment + x_increment + x_increment, y_increment * self.id as f32];
                    bl = [x_increment + x_increment, y_increment * self.id as f32 + y_increment];
                    br = [x_increment + x_increment + x_increment, y_increment * self.id as f32 + y_increment];
                }
            }
        }
        else {
            tl = [0.0, y_increment * self.id as f32];
            tr = [x_increment, y_increment * self.id as f32];
            bl = [0.0, y_increment * self.id as f32 + y_increment];
            br = [x_increment, y_increment * self.id as f32 + y_increment];
        }

        (tr, tl, bl, br)

    }

}

pub struct BlockList {
    pub blocks: Vec<Block>,
    pub atlas: DynamicImage,
}

impl BlockList {

    pub fn initialize() -> BlockList {
        
        let blocks = fs::read_dir("./src/game/data/blocks").unwrap();
        let block_count = fs::read_dir("./src/game/data/blocks").unwrap().count() as u32;
        let mut block_vec: Vec<Block> = Vec::new();

        let mut atlas_buf: image::ImageBuffer<image::Rgba<u8>, Vec<u8>> = image::ImageBuffer::new(64, block_count * 16);
        for (_x, _y, pixel) in atlas_buf.enumerate_pixels_mut() {
            *pixel = image::Rgba([255 as u8, 255 as u8, 255 as u8, 255 as u8]);
        }

        let mut y_offset = 1;
        for block_folder in blocks {
            let block_folder = block_folder.unwrap();
            
            match block_folder.file_type() {
                Ok(file_type) => {
                    if file_type.is_dir() {

                        let dir = fs::read_dir(block_folder.path()).unwrap();
                        println!("registered block: {:?}", block_folder.file_name());
                        let mut sided = true;
                        

                        for path in dir {

                            let file = path.unwrap();
                            let mut image: Option<DynamicImage> = None;
                            let mut x_offset: u32 = 0;

                            if file.file_name() == "texture.png" {
                                image = Some(image::open(file.path()).unwrap());
                                x_offset = 0;
                                sided = false;
                            }
                            else if file.file_name() == "top.png" {
                                image = Some(image::open(file.path()).unwrap());
                                x_offset = 1;
                            }
                            else if file.file_name() == "side.png" {
                                image = Some(image::open(file.path()).unwrap());
                                x_offset = 2;
                            }
                            else if file.file_name() == "bottom.png" {
                                image = Some(image::open(file.path()).unwrap());
                                x_offset = 3;
                            }
                            
                            match image {
                                Some(img) => {
                                    
                                    let offset = cgmath::Vector2::new(x_offset, y_offset) * 16;
                                    for (x, y, pixel) in img.pixels() {
                                        atlas_buf.put_pixel(x + offset.x, y + offset.y, pixel);
                                    }
                                }
                                None => ()
                            }

                        }

                        block_vec.push(Block { id: y_offset as u8, name: block_folder.file_name().to_str().unwrap().into(), sided });
                        y_offset += 1;
                    }
                }
                Err(_) => ()
            }
        }

        
        match atlas_buf.save_with_format("./src/game/data/blocks/atlas.png", image::ImageFormat::Png) {
            Ok(_) => (),
            Err(error) => {
                panic!("Error saving atlas!: {:?}", error);
            }
        }
        
        BlockList {
            blocks: block_vec,
            atlas: image::open("./src/game/data/blocks/atlas.png").unwrap(),
        }
        
    }

}