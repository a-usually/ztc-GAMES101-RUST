use std::collections::HashMap;
use crate::utils::{min, max};
use nalgebra::{Matrix4, Vector3, Vector4, Vector2};
use crate::triangle::Triangle;

const INFINITY: f64 = f64::INFINITY;

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

#[derive(Default, Clone)]
pub struct Rasterizer {
    model: Matrix4<f64>,
    view: Matrix4<f64>,
    projection: Matrix4<f64>,
    jitter: Matrix4<f64>,
    pos_buf: HashMap<usize, Vec<Vector3<f64>>>,
    ind_buf: HashMap<usize, Vec<Vector3<usize>>>,
    col_buf: HashMap<usize, Vec<Vector3<f64>>>,

    frame_buf: Vec<Vector3<f64>>,
    pre_frame_buf: Vec<Vector3<f64>>,
    frame_buf_0: Vec<Vector3<f64>>,

    depth_buf: Vec<f64>,
    /*  You may need to uncomment here to implement the MSAA method  */
    frame_sample: Vec<Vector3<f64>>,
    depth_sample: Vec<f64>,
    num_count: Vec<i32>,
    width: u64,
    height: u64,
    next_id: usize,
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
        r.frame_buf_0.resize((w * h) as usize, Vector3::zeros());

        r.depth_buf.resize((w * h) as usize, INFINITY);
        r.frame_sample.resize((w * h * 4) as usize, Vector3::zeros());
        r.pre_frame_buf.resize((w * h) as usize, Vector3::zeros());
        r.depth_sample.resize((w * h * 4) as usize, INFINITY);
        r.num_count.resize((w * h) as usize, 0);
        r
    }

    fn get_index1(&self, x: usize, y: usize) -> usize {
        ((self.height - 1 - y as u64) * self.width + x as u64) as usize
    }

    fn get_index2(&self, x: usize, y: usize) -> usize {
        ((self.height * 2 - 1 - y as u64) * self.width * 2 + x as u64) as usize
    }

    fn set_pixel(&mut self, point: &Vector3<f64>, color: &Vector3<f64>) {
        let alpha = 0.05;
        let ind = (self.height as f64 - 1.0 - point.y) * self.width as f64 + point.x;
        self.frame_buf[ind as usize] = *color;
        // //no color
        // if self.frame_buf[ind as usize].x == 0.0 && self.frame_buf[ind as usize].y == 0.0 && self.frame_buf[ind as usize].z == 0.0 {
        //     self.frame_buf[ind as usize] = *color;
        // }

        // //have set color
        // else {
        //     self.frame_buf[ind as usize] = self.pre_frame_buf[ind as usize] * (1.0 - alpha) + color * alpha;
        // }
        // self.pre_frame_buf[ind as usize] = self.frame_buf[ind as usize];
    }

    pub fn clear(&mut self, buff: Buffer) {
        match buff {
            Buffer::Color => {
                self.frame_buf.fill(Vector3::new(0.0, 0.0, 0.0));
                self.frame_buf_0.fill(Vector3::new(0.0, 0.0, 0.0));
            }
            Buffer::Depth => {
                self.depth_buf.fill(f64::MAX);
            }
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

    pub fn set_jitter(&mut self, jitter: Matrix4<f64>) {
        self.jitter = jitter;
    }

    fn get_next_id(&mut self) -> usize {
        let res = self.next_id;
        self.next_id += 1;
        res
    }

    pub fn load_position(&mut self, positions: &Vec<Vector3<f64>>) -> PosBufId {
        let id = self.get_next_id();
        self.pos_buf.insert(id, positions.clone());
        PosBufId(id)
    }

    pub fn load_indices(&mut self, indices: &Vec<Vector3<usize>>) -> IndBufId {
        let id = self.get_next_id();
        self.ind_buf.insert(id, indices.clone());
        IndBufId(id)
    }

    pub fn load_colors(&mut self, colors: &Vec<Vector3<f64>>) -> ColBufId {
        let id = self.get_next_id();
        self.col_buf.insert(id, colors.clone());
        ColBufId(id)
    }

    pub fn draw(&mut self, pos_buffer: PosBufId, ind_buffer: IndBufId, col_buffer: ColBufId, _typ: Primitive) {
        let buf = &self.clone().pos_buf[&pos_buffer.0];
        let ind: &Vec<Vector3<usize>> = &self.clone().ind_buf[&ind_buffer.0];
        let col = &self.clone().col_buf[&col_buffer.0];

        let f1 = (50.0 - 0.1) / 2.0;
        let f2 = (50.0 + 0.1) / 2.0;

        let mvp = self.projection * self.view * self.model;
        //let mvp = self.jitter * self.view * self.model;//TAA 

        for i in ind {
            let mut t = Triangle::new();
            let mut v: Vec<nalgebra::Matrix<f64, nalgebra::Const<4>, nalgebra::Const<1>, nalgebra::ArrayStorage<f64, 4, 1>>> =
                vec![mvp * to_vec4(buf[i[0]], Some(1.0)), // homogeneous coordinates
                     mvp * to_vec4(buf[i[1]], Some(1.0)), 
                     mvp * to_vec4(buf[i[2]], Some(1.0))];
    
            for vec in v.iter_mut() {
                *vec = *vec / vec.w;
            }
            for vert in v.iter_mut() {
                vert.x = 0.5 * self.width as f64 * (vert.x + 1.0);
                vert.y = 0.5 * self.height as f64 * (vert.y + 1.0);
                vert.z = vert.z * f1 + f2;
            }
            for j in 0..3 {
                // t.set_vertex(j, Vector3::new(v[j].x, v[j].y, v[j].z));
                t.set_vertex(j, v[j].xyz());
                t.set_vertex(j, v[j].xyz());
                t.set_vertex(j, v[j].xyz());
            }
            let col_x = col[i[0]];
            let col_y = col[i[1]];
            let col_z = col[i[2]];
            t.set_color(0, col_x[0], col_x[1], col_x[2]);
            t.set_color(1, col_y[0], col_y[1], col_y[2]);
            t.set_color(2, col_z[0], col_z[1], col_z[2]);

            self.rasterize_triangle(&t);

            let abs_lumn = 0.005;
            let relative_lumn = 0.001;
            let lumn = Vector3::new(0.299, 0.587, 0.114);//change function
            for x in 0..= self.width as i32 - 1 {
                for y in 0..= self.height as i32 - 1 {
                    let temp = self.get_index1(x as usize, y as usize).clone();
                    let temp1 = if y + 1 <= self.height as i32 - 1 {
                        self.get_index1(x as usize, (y as f64 + 1.0) as usize).clone()
                    }
                    else {
                        self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp].clone());
                        continue
                    };

                    let temp2 = if x + 1 <= self.width as i32 -1 {
                        self.get_index1((x as f64 + 1.0) as usize, y as usize).clone()
                    }
                    else {
                        self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp].clone());
                        continue
                    };

                    let temp3 = if y - 1 >= 0 {
                        self.get_index1(x as usize, (y as f64 - 1.0) as usize).clone()
                    }
                    else {
                        self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp].clone());
                        continue
                    };

                    let temp4 = if x - 1 >= 0 {
                        self.get_index1((x as f64 - 1.0) as usize, y as usize).clone()
                    }
                    else {
                        self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp].clone());
                        continue
                    };
                    //coordinate

                    let n_rgb = self.frame_buf_0[temp1];
                    let e_rgb = self.frame_buf_0[temp2];
                    let s_rgb = self.frame_buf_0[temp3];
                    let w_rgb = self.frame_buf_0[temp4];

                    let m_rgb = self.frame_buf_0[temp];

                    let n_lumn:f64 = n_rgb.dot(&lumn);
                    let e_lumn:f64 = e_rgb.dot(&lumn);
                    let s_lumn:f64 = s_rgb.dot(&lumn);
                    let w_lumn:f64 = w_rgb.dot(&lumn);
                    let m_lumn:f64 = m_rgb.dot(&lumn);

                    let lumnmin = min(n_lumn, min(e_lumn, min(s_lumn, w_lumn)));
                    let lumnmax = max(n_lumn, max(e_lumn, max(s_lumn, w_lumn)));
                    let lumncompare = lumnmax - lumnmin;
                    let isedge:bool = lumncompare > max(abs_lumn, relative_lumn * lumnmax);//the max limit

                    if isedge {
                        //println!("ss");
                        let lumn_grad_n = n_lumn - m_lumn;
                        let lumn_grad_s = s_lumn - m_lumn;
                        let lumn_grad_e = e_lumn - m_lumn;
                        let lumn_grad_w = w_lumn - m_lumn;//the grad
                        let lumn_grad_1 = (lumn_grad_n + lumn_grad_s).abs();
                        let lumn_grad_2 = (lumn_grad_e + lumn_grad_w).abs();
                        let ishorz = lumn_grad_1 > lumn_grad_2;

                        let mut normal = Vector2::new(0.0, 0.0);

                        if ishorz {
                            normal.y = (lumn_grad_n.abs() - lumn_grad_s.abs()).signum();
                        }
                        else {
                            normal.x = (lumn_grad_e.abs() - lumn_grad_w.abs()).signum();
                        }

                        let lumn_ave = (e_lumn + n_lumn + w_lumn + s_lumn) * 0.25;
                        let lumn_ml = (m_lumn - lumn_ave).abs();

                        if lumn_ml == 0.0 {
                            self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp].clone());
                            continue;
                        }
                        else {
                            let modify = lumn_ml / lumncompare;
                            let x_0 = min(x as f64 + modify * normal.x, self.width as f64 - 1.0) as usize;
                            let y_0 = min(y as f64 + modify * normal.y, self.height as f64 -1.0) as usize;
                            let temp_00 = self.get_index1(x_0, y_0);
                            self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp_00].clone());
                        }
                    }
                    else {
                        self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp].clone());
                        continue;
                    }
                }
            }

        }
    }

    pub fn rasterize_triangle(&mut self, t: &Triangle) {
        let abs_lumn = 0.005;
        let relative_lumn = 0.001;
        let lumn = Vector3::new(0.299, 0.587, 0.114);//change function

        /*  implement your code here  */
        for x in 0..= self.width as i32 - 1 {
            for y in 0..= self.height as i32 - 1 {
                if inside_triangle(x as f64 + 0.5, y as f64 + 0.5, &t.v) && (t.v[0].z < self.depth_buf[self.get_index1(x as usize, y as usize)]) {
                    let temp = self.get_index1(x as usize, y as usize).clone();
                    self.depth_buf[temp] = t.v[0].z;
                    self.frame_buf_0[temp] = t.get_color().clone();
                }
            }
        }

        // for x in 0..= self.width as i32 - 1 {
        //     for y in 0..= self.height as i32 - 1 {
                    // let temp = self.get_index1(x as usize, y as usize).clone();
                    // let temp1 = if y + 1 <= self.height as i32 - 1 {
                    //     self.get_index1(x as usize, (y as f64 + 1.0) as usize).clone()
                    // }
                    // else {
                    //     self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp].clone());
                    //     continue
                    // };

                    // let temp2 = if x + 1 <= self.width as i32 -1 {
                    //     self.get_index1((x as f64 + 1.0) as usize, y as usize).clone()
                    // }
                    // else {
                    //     self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp].clone());
                    //     continue
                    // };

                    // let temp3 = if y >= 0 {
                    //     self.get_index1((x + 1) as usize, (y + 1) as usize).clone()
                    // }
                    // else {
                    //     self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp].clone());
                    //     continue
                    // };

                    // let temp4 = if x  >= 0 {
                    //     self.get_index1(x as usize, y as usize).clone()
                    // }
                    // else {
                    //     self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp].clone());
                    //     continue
                    // };
                    //coordinate

                    // let se_rgb = self.frame_buf_0[temp1];
                    // let nw_rgb = self.frame_buf_0[temp2];
                    // let ne_rgb = self.frame_buf_0[temp3];
                    // let sw_rgb = self.frame_buf_0[temp4];

                    // let m_rgb = self.frame_buf_0[temp];

                    // let ne_lumn:f64 = ne_rgb.dot(&lumn);
                    // let nw_lumn:f64 = nw_rgb.dot(&lumn);
                    // let se_lumn:f64 = se_rgb.dot(&lumn);
                    // let sw_lumn:f64 = sw_rgb.dot(&lumn);
                    // let m_lumn:f64 = m_rgb.dot(&lumn);

                    // let lumnmin = min(n_lumn, min(e_lumn, min(s_lumn, w_lumn)));
                    // let lumnmax = max(nw_lumn, max(ne_lumn, max(se_lumn, sw_lumn)));
                    // let lumncompare = lumnmax - lumnmin;
                    // let isedge:bool = lumncompare > max(abs_lumn, relative_lumn * lumnmax);//the max limit

                    // if isedge {
                        //println!("ss");
                        // let lumn_grad_n = n_lumn - m_lumn;
                        // let lumn_grad_s = s_lumn - m_lumn;
                        // let lumn_grad_e = e_lumn - m_lumn;
                        // let lumn_grad_w = w_lumn - m_lumn;//the grad
                        // let lumn_grad_1 = (lumn_grad_n + lumn_grad_s).abs();
                        // let lumn_grad_2 = (lumn_grad_e + lumn_grad_w).abs();
                        // let ishorz = lumn_grad_1 > lumn_grad_2;

                    //     let mut normal = Vector2::new(0.0, 0.0);

                    //     if ishorz {
                    //         normal.y = (lumn_grad_n.abs() - lumn_grad_s.abs()).signum();
                    //     }
                    //     else {
                    //         normal.x = (lumn_grad_e.abs() - lumn_grad_w.abs()).signum();
                    //     }

                    //     let lumn_ave = (e_lumn + n_lumn + w_lumn + s_lumn) * 0.25;
                    //     let lumn_ml = (m_lumn - lumn_ave).abs();

                    //     if lumn_ml == 0.0 {
                    //         self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp].clone());
                    //         continue;
                    //     }
                    //     else {
                    //         let modify = lumn_ml / lumncompare;
                    //         let x_0 = min(x as f64 + modify * normal.x, self.width as f64 - 1.0) as usize;
                    //         let y_0 = min(y as f64 + modify * normal.y, self.height as f64 -1.0) as usize;
                    //         let temp_00 = self.get_index1(x_0, y_0);
                    //         self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp_00].clone());
                    //     }
                    // }
                    // else {
                    //     self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &self.frame_buf_0[temp].clone());
                    //     continue;
                    // }

                    // let mut dir = Vector2::new(0.0, 0.0);
                    // dir.x = sw_lumn + se_lumn - nw_lumn - ne_lumn;
                    // dir.y = nw_lumn + sw_lumn - ne_lumn - se_lumn;
        //     }
        // }

    //     for x in 0..= self.width as i32 - 1 {
    //         for y in 0..= self.height as i32 - 1 {
    //             let temp_0 = self.get_index1(x as usize, y as usize).clone();
    //             if inside_triangle(x as f64 + 0.25, y as f64 + 0.25, &t.v) && (t.v[0].z < self.depth_sample[self.get_index2((x * 2) as usize, (y * 2) as usize)]) {
    //                 let temp = self.get_index2((x * 2) as usize, (y * 2) as usize).clone();
    //                 self.depth_sample[temp] = t.v[0].z;
    //                 self.frame_sample[temp] = t.get_color();
    //                 self.num_count[temp_0] += 1;
    //             }

    //             if inside_triangle(x as f64 + 0.75, y as f64 + 0.25, &t.v) && (t.v[0].z < self.depth_sample[self.get_index2((x * 2 + 1) as usize, (y * 2) as usize)]) {
    //                 let temp = self.get_index2((x * 2 + 1) as usize, (y * 2) as usize).clone();
    //                 self.depth_sample[temp] = t.v[0].z;
    //                 self.frame_sample[temp] = t.get_color();
    //                 self.num_count[temp_0] += 1;
    //             }

    //             if inside_triangle(x as f64 + 0.25, y as f64 + 0.75, &t.v) && (t.v[0].z < self.depth_sample[self.get_index2((x * 2) as usize, (y * 2 + 1) as usize)]) {
    //                 let temp = self.get_index2((x * 2) as usize, (y * 2 + 1) as usize).clone();
    //                 self.depth_sample[temp] = t.v[0].z;
    //                 self.frame_sample[temp] = t.get_color();
    //                 self.num_count[temp_0] += 1;
    //             }

    //             if inside_triangle(x as f64 + 0.75, y as f64 + 0.75, &t.v) && (t.v[0].z < self.depth_sample[self.get_index2((x * 2 + 1) as usize, (y * 2 + 1) as usize)]) {
    //                 let temp = self.get_index2((x * 2 + 1) as usize, (y * 2 + 1) as usize).clone();
    //                 self.depth_sample[temp] = t.v[0].z;
    //                 self.frame_sample[temp] = t.get_color();
    //                 self.num_count[temp_0] += 1;
    //             }
    //         }
    //     }

    //     for x in 0..=self.width as i32 - 1 {
    //         for y in 0..=self.height as i32 - 1 {
    //             let temp = self.get_index1(x as usize, y as usize).clone();
    //             if self.num_count[temp] > 0 && (t.v[0].z < self.depth_buf[self.get_index1(x as usize, y as usize)]) {
    //                 self.depth_buf[temp] = t.v[0].z;
    //                 let mut color_temp = Vector3::new(0.0, 0.0, 0.0);
    //                 let temp1 = self.get_index2((x * 2) as usize, (y * 2) as usize).clone();
    //                 let temp2 = self.get_index2((x * 2 + 1) as usize, (y * 2) as usize).clone();
    //                 let temp3 = self.get_index2((x * 2) as usize, (y * 2 + 1) as usize).clone();
    //                 let temp4 = self.get_index2((x * 2 + 1) as usize, (y * 2 + 1) as usize).clone();

    //                 color_temp.x = (self.frame_sample[temp1].x + self.frame_sample[temp2].x + self.frame_sample[temp3].x + self.frame_sample[temp4].x) / 4.0;
    //                 color_temp.y = (self.frame_sample[temp1].y + self.frame_sample[temp2].y + self.frame_sample[temp3].y + self.frame_sample[temp4].y) / 4.0;
    //                 color_temp.z = (self.frame_sample[temp1].z + self.frame_sample[temp2].z + self.frame_sample[temp3].z + self.frame_sample[temp4].z) / 4.0;

    //                 self.set_pixel(&Vector3::new(x as f64, y as f64, 0.0), &color_temp);
    //             }
    //         }
    //     }
    }

    pub fn frame_buffer(&self) -> &Vec<Vector3<f64>> {
        &self.frame_buf
    }
}
    fn to_vec4(v3: Vector3<f64>, w: Option<f64>) -> Vector4<f64> {
        Vector4::new(v3.x, v3.y, v3.z, w.unwrap_or(1.0))
    }

    fn inside_triangle(x: f64, y: f64, v: &[Vector3<f64>; 3]) -> bool {
        /*  implement your code here  */
        let mut v_0 = v.clone();

    //make sure it is clockwise
    if (v_0[1].x - v_0[0].x) * (v_0[2].y - v_0[0].y) - (v_0[1].y - v_0[0].y)*(v_0[2].x - v_0[0].x) < 0.0 {
        v_0[0] = v[1].clone();
        v_0[1] = v[0].clone();
    }

    //compute the cross product
    let v0_v1 = (v_0[1].x - v_0[0].x) * (y - v_0[0].y) - (v_0[1].y - v_0[0].y) * (x - v_0[0].x);
	let v1_v2 = (v_0[2].x - v_0[1].x) * (y - v_0[1].y) - (v_0[2].y - v_0[1].y) * (x - v_0[1].x);
	let v2_v0 = (v_0[0].x - v_0[2].x) * (y - v_0[2].y) - (v_0[0].y - v_0[2].y) * (x - v_0[2].x);

	if v0_v1 > 0.0 && v1_v2 > 0.0 && v2_v0 > 0.0 {
        return true;
    }

    false
}

fn compute_barycentric2d(x: f64, y: f64, v: &[Vector3<f64>; 3]) -> (f64, f64, f64) {
    let c1 = (x * (v[1].y - v[2].y) + (v[2].x - v[1].x) * y + v[1].x * v[2].y - v[2].x * v[1].y)
        / (v[0].x * (v[1].y - v[2].y) + (v[2].x - v[1].x) * v[0].y + v[1].x * v[2].y - v[2].x * v[1].y);
    let c2 = (x * (v[2].y - v[0].y) + (v[0].x - v[2].x) * y + v[2].x * v[0].y - v[0].x * v[2].y)
        / (v[1].x * (v[2].y - v[0].y) + (v[0].x - v[2].x) * v[1].y + v[2].x * v[0].y - v[0].x * v[2].y);
    let c3 = (x * (v[0].y - v[1].y) + (v[1].x - v[0].x) * y + v[0].x * v[1].y - v[1].x * v[0].y)
        / (v[2].x * (v[0].y - v[1].y) + (v[1].x - v[0].x) * v[2].y + v[0].x * v[1].y - v[1].x * v[0].y);
    (c1, c2, c3)
}

