use nalgebra::{Vector3};
use opencv::core::{MatTraitConst, VecN};
use opencv::imgcodecs::{imread, IMREAD_COLOR};
use opencv::prelude::FacemarkAAM_ConfigTraitConst;
use crate::utils::{min,max};

#[derive(Clone)]
pub struct Texture {
    pub img_data: opencv::core::Mat,
    pub width: usize,
    pub height: usize,
}

impl Texture {
    pub fn new(name: &str) -> Self {
        let img_data = imread(name, IMREAD_COLOR).expect("Image reading error!");
        let width = img_data.cols() as usize;
        let height = img_data.rows() as usize;
        Texture {
            img_data,
            width,
            height,
        }
    }

    // pub fn get_color(&self, mut u: f64, mut v: f64) -> Vector3<f64> {
    //     if u < 0.0 { u = 0.0; }
    //     if u > 1.0 { u = 1.0; }
    //     if v < 0.0 { v = 0.0; }
    //     if v > 1.0 { v = 1.0; }

    //     let u_img = u * self.width as f64;
    //     let v_img = (1.0 - v) * self.height as f64;

    //     let color: &VecN<u8, 3> = self.img_data.at_2d(v_img as i32, u_img as i32).unwrap();

    //     Vector3::new(color[2] as f64, color[1] as f64, color[0] as f64)
    // }

    pub fn getColorBilinear(&self, mut u: f64, mut v: f64) -> Vector3<f64> {
        // 在此实现双线性插值函数
        if u < 0.0 { u = 0.0; }
        if u > 1.0 { u = 1.0; }
        if v < 0.0 { v = 0.0; }
        if v > 1.0 { v = 1.0; }

        let u_img = u * self.width as f64;
        let v_img = (1.0 - v) * self.height as f64;

        let u00 = u_img as i32;
        let v00 = v_img as i32;

        let s = u_img - u00 as f64;
        let t = v_img - v00 as f64;

        let color00: &VecN<u8, 3> = self.img_data.at_2d(v00 as i32, u00 as i32).unwrap();
        let color01: &VecN<u8, 3> = self.img_data.at_2d(v00 as i32, min(u00 as f64 + 1.0, 1023.0) as i32).unwrap();
        let color10: &VecN<u8, 3> = self.img_data.at_2d(min(v00 as f64 + 1.0, 1023.0) as i32, u00).unwrap();
        let color11: &VecN<u8, 3> = self.img_data.at_2d(min(v00 as f64 + 1.0, 1023.0) as i32, min(u00 as f64 + 1.0, 1023.0) as i32).unwrap();

        let color0 = lerp1(t, &color00, &color10);
        let color1 = lerp1(t, &color01, &color11);


        let color = lerp2(s, &color0, &color1);

        Vector3::new(color[2] as f64, color[1] as f64, color[0] as f64)
    }
}

pub fn lerp1(x: f64, v1: &VecN<u8, 3>, v2: &VecN<u8, 3>) -> VecN<f64, 3> {
    let mut temp:VecN<f64, 3> = VecN([0.0 as f64; 3]);
    temp[0] = (v1[0] as f64 + (v2[0] as f64 - v1[0] as f64) * x) as f64;
    temp[1] = (v1[1] as f64 + (v2[1] as f64 - v1[1] as f64) * x) as f64;
    temp[2] = (v1[2] as f64 + (v2[2] as f64 - v1[2] as f64) * x) as f64;

    temp
}

pub fn lerp2(x: f64, v1: &VecN<f64, 3>, v2: &VecN<f64, 3>) -> VecN<f64, 3> {
    let mut temp:VecN<f64, 3> = VecN([0.0 as f64; 3]);
    temp[0] = (v1[0] as f64 + (v2[0] as f64 - v1[0] as f64) * x);
    temp[1] = (v1[1] as f64 + (v2[1] as f64 - v1[1] as f64) * x);
    temp[2] = (v1[2] as f64 + (v2[2] as f64 - v1[2] as f64) * x);

    temp
}