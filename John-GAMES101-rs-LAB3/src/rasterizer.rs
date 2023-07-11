use std::rc::Rc;

use nalgebra::{Matrix4, Vector2, Vector3, Vector4};
use opencv::prelude::{FacemarkAAM_ModelTraitConst, FacemarkAAM_ParamsTraitConst};
use crate::shader::{FragmentShaderPayload, VertexShaderPayload};
use crate::texture::Texture;
use crate::triangle::{Triangle, self};
use crate::utils::{min, max};

#[allow(dead_code)]
pub enum Buffer {
    Color,
    Depth,
    Both,
}

#[allow(dead_code)]
pub enum Primitive {
    Line,
    Triangle,
}

#[derive(Default)]
pub struct Rasterizer {
    model: Matrix4<f64>,
    view: Matrix4<f64>,
    projection: Matrix4<f64>,
    texture: Option<Texture>,

    vert_shader: Option<fn(&VertexShaderPayload) -> Vector3<f64>>,
    fragment_shader: Option<fn(&FragmentShaderPayload) -> Vector3<f64>>,
    frame_buf: Vec<Vector3<f64>>,
    depth_buf: Vec<f64>,
    width: u64,
    height: u64,
}

#[derive(Clone, Copy)]
pub struct PosBufId(usize);

#[derive(Clone, Copy)]
pub struct IndBufId(usize);

#[derive(Clone, Copy)]
pub struct ColBufId(usize);

impl Rasterizer {
    pub fn new(w: u64, h: u64) -> Self {
        let mut r = Rasterizer::default();
        r.width = w;
        r.height = h;
        r.frame_buf.resize((w * h) as usize, Vector3::zeros());
        r.depth_buf.resize((w * h) as usize, 0.0);
        r.texture = None;
        r
    }

    pub fn get_index1(height: u64, width: u64, x: usize, y: usize) -> usize {
        ((height - 1 - y as u64) * width + x as u64) as usize
    }

    pub fn get_index2(height: u64, width: u64, x: usize, y: usize) -> usize {
        ((height * 2 - 1 - y as u64) * width * 2 + x as u64) as usize
    }

    fn set_pixel(height: u64, width: u64, frame_buf: &mut Vec<Vector3<f64>>, point: &Vector3<f64>, color: &Vector3<f64>) {
        let ind = (height as f64 - 1.0 - point.y) * width as f64 + point.x;
        frame_buf[ind as usize] = *color;
    }

    pub fn clear(&mut self, buff: Buffer) {
        match buff {
            Buffer::Color =>
                self.frame_buf.fill(Vector3::new(0.0, 0.0, 0.0)),
            Buffer::Depth =>
                self.depth_buf.fill(f64::MAX),
            Buffer::Both => {
                self.frame_buf.fill(Vector3::new(0.0, 0.0, 0.0));
                self.depth_buf.fill(f64::MAX);
            }
        }
    }
    pub fn set_model(&mut self, model: Matrix4<f64>) {
        self.model = model;
    }

    pub fn set_view(&mut self, view: Matrix4<f64>) {
        self.view = view;
    }

    pub fn set_projection(&mut self, projection: Matrix4<f64>) {
        self.projection = projection;
    }

    pub fn set_texture(&mut self, tex: Texture) { 
        self.texture = Some(tex); 
    }

    pub fn set_vertex_shader(&mut self, vert_shader: fn(&VertexShaderPayload) -> Vector3<f64>) {
        self.vert_shader = Some(vert_shader);
    }
    
    pub fn set_fragment_shader(&mut self, frag_shader: fn(&FragmentShaderPayload) -> Vector3<f64>) {
        self.fragment_shader = Some(frag_shader);
    }

    pub fn draw(&mut self, triangles: &Vec<Triangle>) {
        let mvp = self.projection * self.view * self.model;

        // 遍历每个小三角形
        for triangle in triangles { 
            self.rasterize_triangle(&triangle, mvp); 
        }
    }

    pub fn rasterize_triangle(&mut self, triangle: &Triangle, mvp: Matrix4<f64>) {
        /*  Implement your code here  */
        let (new_tri, view_space_pos) = Self::get_new_tri(triangle, self.view, self.model, mvp,(self.width, self.height));

        let x_min = min(self.width as f64 - 1.0, min(new_tri.v[0].x, min(new_tri.v[1].x, new_tri.v[2].x))) as i32;
        let x_max = min(self.width as f64 - 1.0, max(new_tri.v[0].x, max(new_tri.v[1].x, new_tri.v[2].x))) as i32;
        let y_min = min(self.height as f64 - 1.0, min(new_tri.v[0].y, min(new_tri.v[1].y, new_tri.v[2].y))) as i32;
        let y_max = min(self.height as f64 - 1.0, max(new_tri.v[0].y, max(new_tri.v[1].y, new_tri.v[2].y))) as i32;

        for x in x_min..= x_max {
            for y in y_min..= y_max {
                if inside_triangle(x as f64 + 0.5, y as f64 + 0.5, &new_tri.v) && (new_tri.v[0].z <= self.depth_buf[Self::get_index1(self.height, self.width, x as usize, y as usize)]) {
                    let (a, b, c) = compute_barycentric2d_1(x as f64, y as f64, &new_tri.v);
                    let temp = Self::get_index1(self.height, self.width, x as usize, y as usize).clone();
                    self.depth_buf[temp] = new_tri.v[0].z;

                    let temp_color = Self::interpolate_vec3(a, b, c, new_tri.color[0], new_tri.color[1], new_tri.color[2],  1.0);
                    let temp_normal = Self::interpolate_vec3(a, b, c, new_tri.normal[0], new_tri.normal[1], new_tri.normal[2], 1.0);
                    let temp_texcoord = Self::interpolate_vec2(a, b, c, new_tri.tex_coords[0], new_tri.tex_coords[1], new_tri.tex_coords[2], 1.0);
                    let temp_tex = self.texture.clone().unwrap();

                    let temp_0 = FragmentShaderPayload::new(&temp_color, &temp_normal, &temp_texcoord, Some(Rc::new(&temp_tex)));
                    Self::set_pixel(self.height, self.width, &mut self.frame_buf, &Vector3::new(x as f64, y as f64, 0.0), &self.fragment_shader.unwrap()(&temp_0));
                    //Self::set_pixel(self.height, self.width, &mut self.frame_buf, &Vector3::new(x as f64, y as f64, 0.0), &(temp_color * 255.0));
                    //self.frame_buf_0[temp] = t.get_color().clone();
                    //
                }
            }
        }
    }
    
    pub fn interpolate_vec3(a: f64, b: f64, c: f64, vert1: Vector3<f64>, vert2: Vector3<f64>, vert3: Vector3<f64>, weight: f64) -> Vector3<f64> {
        (a * vert1 + b * vert2 + c * vert3) / weight
    }
    pub fn interpolate_vec2(a: f64, b: f64, c: f64, vert1: Vector2<f64>, vert2: Vector2<f64>, vert3: Vector2<f64>, weight: f64) -> Vector2<f64> {
        (a * vert1 + b * vert2 + c * vert3) / weight
    }

    fn get_new_tri(t: &Triangle, view: Matrix4<f64>, model: Matrix4<f64>, mvp: Matrix4<f64>,
                    (width, height): (u64, u64)) -> (Triangle, Vec<Vector3<f64>>) {
        let f1 = (50.0 - 0.1) / 2.0; // zfar和znear距离的一半
        let f2 = (50.0 + 0.1) / 2.0; // zfar和znear的中心z坐标
        let mut new_tri = (*t).clone();
        let mm: Vec<Vector4<f64>> = (0..3).map(|i| view * model * t.v[i]).collect();
        let view_space_pos: Vec<Vector3<f64>> = mm.iter().map(|v| v.xyz()).collect();
        let mut v: Vec<Vector4<f64>> = (0..3).map(|i| mvp * t.v[i]).collect();

        // 换算齐次坐标
        for vec in v.iter_mut() {
            vec.x /= vec.w;
            vec.y /= vec.w;
            vec.z /= vec.w;
        }
        let inv_trans = (view * model).try_inverse().unwrap().transpose();
        let n: Vec<Vector4<f64>> = (0..3).map(|i| inv_trans * to_vec4(t.normal[i], Some(0.0))).collect();

        // 视口变换得到顶点在屏幕上的坐标, 即screen space
        for vert in v.iter_mut() {
            vert.x = 0.5 * width as f64 * (vert.x + 1.0);
            vert.y = 0.5 * height as f64 * (vert.y + 1.0);
            vert.z = vert.z * f1 + f2;
        }
        for i in 0..3 {
            new_tri.set_vertex(i, v[i]);
        }
        for i in 0..3 {
            new_tri.set_normal(i, n[i].xyz());
        }

        new_tri.set_color(0, 148.0, 121.0, 92.0);
        new_tri.set_color(1, 148.0, 121.0, 92.0);
        new_tri.set_color(2, 148.0, 121.0, 92.0);

        (new_tri, view_space_pos)
    }

    pub fn frame_buffer(&self) -> &Vec<Vector3<f64>> {
        &self.frame_buf
    }

}

fn to_vec4(v3: Vector3<f64>, w: Option<f64>) -> Vector4<f64> {
    Vector4::new(v3.x, v3.y, v3.z, w.unwrap_or(1.0))
}

fn inside_triangle(x: f64, y: f64, v: &[Vector4<f64>; 3]) -> bool {
    let v = [
        Vector3::new(v[0].x, v[0].y, 1.0),
        Vector3::new(v[1].x, v[1].y, 1.0),
        Vector3::new(v[2].x, v[2].y, 1.0), ];

    let f0 = v[1].cross(&v[0]);
    let f1 = v[2].cross(&v[1]);
    let f2 = v[0].cross(&v[2]);
    let p = Vector3::new(x, y, 1.0);
    if (p.dot(&f0) * f0.dot(&v[2]) > 0.0) &&
        (p.dot(&f1) * f1.dot(&v[0]) > 0.0) &&
        (p.dot(&f2) * f2.dot(&v[1]) > 0.0) {
        true
    } else {
        false
    }
}

pub fn compute_barycentric2d_1(x: f64, y: f64, v: &[Vector4<f64>; 3]) -> (f64, f64, f64) {
    let c1 = (x * (v[1].y - v[2].y) + (v[2].x - v[1].x) * y + v[1].x * v[2].y - v[2].x * v[1].y) / (v[0].x * (v[1].y - v[2].y) + (v[2].x - v[1].x) * v[0].y + v[1].x * v[2].y - v[2].x * v[1].y);
    let c2 = (x * (v[2].y - v[0].y) + (v[0].x - v[2].x) * y + v[2].x * v[0].y - v[0].x * v[2].y) / (v[1].x * (v[2].y - v[0].y) + (v[0].x - v[2].x) * v[1].y + v[2].x * v[0].y - v[0].x * v[2].y);
    let c3 = (x * (v[0].y - v[1].y) + (v[1].x - v[0].x) * y + v[0].x * v[1].y - v[1].x * v[0].y) / (v[2].x * (v[0].y - v[1].y) + (v[1].x - v[0].x) * v[2].y + v[0].x * v[1].y - v[1].x * v[0].y);
    (c1, c2, c3)
}

pub fn compute_barycentric2d_2(x: f64, y: f64, v: &[Vector3<f64>; 3]) -> (f64, f64, f64) {
    let c1 = (x * (v[1].y - v[2].y) + (v[2].x - v[1].x) * y + v[1].x * v[2].y - v[2].x * v[1].y) / (v[0].x * (v[1].y - v[2].y) + (v[2].x - v[1].x) * v[0].y + v[1].x * v[2].y - v[2].x * v[1].y);
    let c2 = (x * (v[2].y - v[0].y) + (v[0].x - v[2].x) * y + v[2].x * v[0].y - v[0].x * v[2].y) / (v[1].x * (v[2].y - v[0].y) + (v[0].x - v[2].x) * v[1].y + v[2].x * v[0].y - v[0].x * v[2].y);
    let c3 = (x * (v[0].y - v[1].y) + (v[1].x - v[0].x) * y + v[0].x * v[1].y - v[1].x * v[0].y) / (v[2].x * (v[0].y - v[1].y) + (v[1].x - v[0].x) * v[2].y + v[0].x * v[1].y - v[1].x * v[0].y);
    (c1, c2, c3)
}