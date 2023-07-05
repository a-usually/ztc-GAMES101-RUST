use std::os::raw::c_void;
use nalgebra::{Matrix4, Vector3};
use opencv::core::{Mat, MatTraitConst};
use opencv::imgproc::{COLOR_RGB2BGR, cvt_color};

pub type V3d = Vector3<f64>;

pub(crate) fn get_view_matrix(eye_pos: V3d) -> Matrix4<f64> {
    let mut view: Matrix4<f64> = Matrix4::identity();
    /*  implement your code here  */

    let mut tview: Matrix4<f64> = Matrix4::identity();
    tview[(0, 0)] = 1.0;
    tview[(0, 1)] = 0.0;
    tview[(0, 2)] = 0.0;
    tview[(0, 3)] = -eye_pos.x;

    tview[(1, 0)] = 0.0;
    tview[(1, 1)] = 1.0;
    tview[(1, 2)] = 0.0;
    tview[(1, 3)] = -eye_pos.y;

    tview[(2, 0)] = 0.0;
    tview[(2, 1)] = 0.0;
    tview[(2, 2)] = 1.0;
    tview[(2, 3)] = -eye_pos.z;

    tview[(3, 0)] = 0.0;
    tview[(3, 1)] = 0.0;
    tview[(3, 2)] = 0.0;
    tview[(3, 3)] = 1.0;

    view = tview;

    view
}

pub(crate) fn get_model_matrix(rotation_angle: f64) -> Matrix4<f64> {
    let mut model: Matrix4<f64> = Matrix4::identity();
    /*  implement your code here  */
    model[(0, 0)] = rotation_angle.to_radians().cos();
    model[(0, 1)] = -rotation_angle.to_radians().sin();
    model[(0, 2)] = 0.0;
    model[(0, 3)] = 0.0;

    model[(1, 0)] = rotation_angle.to_radians().sin();
    model[(1, 1)] = rotation_angle.to_radians().cos();
    model[(1, 2)] = 0.0;
    model[(1, 3)] = 0.0;

    model[(2, 0)] = 0.0;
    model[(2, 1)] = 0.0;
    model[(2, 2)] = 1.0;
    model[(2, 3)] = 0.0;

    model[(3, 0)] = 0.0;
    model[(3, 1)] = 0.0;
    model[(3, 2)] = 0.0;
    model[(3, 3)] = 1.0;

    model
}

pub(crate) fn get_projection_matrix(eye_fov: f64, aspect_ratio: f64, z_near: f64, z_far: f64) -> Matrix4<f64> {
    let mut projection: Matrix4<f64> = Matrix4::identity();
    /*  implement your code here  */
    let t = z_near * (eye_fov / 2.0).tan();
    let r = t * aspect_ratio;
    let l = -r;
    let b = -t;

    let mut persp: Matrix4<f64> = Matrix4::identity();
    persp[(0, 0)] = z_near;
    persp[(0, 1)] = 0.0;
    persp[(0, 2)] = 0.0;
    persp[(0, 3)] = 0.0;

    persp[(1, 0)] = 0.0;
    persp[(1, 1)] = z_near;
    persp[(1, 2)] = 0.0;
    persp[(1, 3)] = 0.0;

    persp[(2, 0)] = 0.0;
    persp[(2, 1)] = 0.0;
    persp[(2, 2)] = z_near + z_far;
    persp[(2, 3)] = -z_far * z_near;

    persp[(3, 0)] = 0.0;
    persp[(3, 1)] = 0.0;
    persp[(3, 2)] = 1.0;
    persp[(3, 3)] = 0.0;

    let mut ortho1: Matrix4<f64> = Matrix4::identity();
    ortho1[(0, 0)] = 2.0 / (r - l);
    ortho1[(0, 1)] = 0.0;
    ortho1[(0, 2)] = 0.0;
    ortho1[(0, 3)] = 0.0;

    ortho1[(1, 0)] = 0.0;
    ortho1[(1, 1)] = 2.0 / (t - b);
    ortho1[(1, 2)] = 0.0;
    ortho1[(1, 3)] = 0.0;

    ortho1[(2, 0)] = 0.0;
    ortho1[(2, 1)] = 0.0;
    ortho1[(2, 2)] = 2.0 / (z_near - z_far);
    ortho1[(2, 3)] = 0.0;

    ortho1[(3, 0)] = 0.0;
    ortho1[(3, 1)] = 0.0;
    ortho1[(3, 2)] = 0.0;
    ortho1[(3, 3)] = 1.0;

    let mut ortho2: Matrix4<f64> = Matrix4::identity();
    ortho2[(0, 0)] = 1.0;
    ortho2[(0, 1)] = 0.0;
    ortho2[(0, 2)] = 0.0;
    ortho2[(0, 3)] = (r + l) / -2.0;

    ortho2[(1, 0)] = 0.0;
    ortho2[(1, 1)] = 1.0;
    ortho2[(1, 2)] = 0.0;
    ortho2[(1, 3)] = (t + b) / -2.0;

    ortho2[(2, 0)] = 0.0;
    ortho2[(2, 1)] = 0.0;
    ortho2[(2, 2)] = 1.0;
    ortho2[(2, 3)] = (z_far + z_near) / -2.0;

    ortho2[(3, 0)] = 0.0;
    ortho2[(3, 1)] = 0.0;
    ortho2[(3, 2)] = 0.0;
    ortho2[(3, 3)] = 1.0;

    projection = ortho1 * ortho2 * persp;

    projection
}

pub fn get_rotation_0(axis: Vector3<f64>) -> Matrix4<f64> {
    let angle: f64 = 0.0;
    let mut model:Matrix4<f64> = Matrix4::identity();

    let mut vec_n:Matrix4<f64> = Matrix4::identity();
    vec_n[(0, 0)] = axis.x * axis.x;
    vec_n[(0, 1)] = axis.x * axis.y;
    vec_n[(0, 2)] = axis.x * axis.z;
    vec_n[(0, 3)] = 0.0;

    vec_n[(1, 0)] = axis.x * axis.y;
    vec_n[(1, 1)] = axis.y * axis.y;
    vec_n[(1, 2)] = axis.y * axis.z;
    vec_n[(1, 3)] = 0.0;

    vec_n[(2, 0)] = axis.z * axis.x;
    vec_n[(2, 1)] = axis.z * axis.y;
    vec_n[(2, 2)] = axis.z * axis.z;
    vec_n[(2, 3)] = 0.0;
//add
    vec_n[(3, 0)] = 0.0;
    vec_n[(3, 1)] = 0.0;
    vec_n[(3, 2)] = 0.0;
    vec_n[(3, 3)] = 1.0;

    let mut sub:Matrix4<f64> = Matrix4::identity();

    sub[(0, 0)] = 0.0;
    sub[(0, 1)] = -axis.z;
    sub[(0, 2)] = axis.y;
    sub[(0, 3)] = 0.0;

    sub[(1, 0)] = axis.z;
    sub[(1, 1)] = 0.0;
    sub[(1, 2)] = -axis.x;
    sub[(1, 3)] = 0.0;

    sub[(2, 0)] = -axis.y;
    sub[(2, 1)] = axis.x;
    sub[(2, 2)] = 0.0;
    sub[(2, 3)] = 0.0;

    sub[(3, 0)] = 0.0;
    sub[(3, 1)] = 0.0;
    sub[(3, 2)] = 0.0;
    sub[(3, 3)] = 0.0;

    model = model * angle.to_radians().cos() + vec_n * (1.0 - angle.to_radians().cos()) + sub * angle.to_radians().sin();

    model
}

pub fn get_rotation(axis: Vector3<f64>, angle: f64) -> Matrix4<f64> {
    let mut model:Matrix4<f64> = Matrix4::identity();
    
    let mut vec_n:Matrix4<f64> = Matrix4::identity();
    vec_n[(0, 0)] = axis.x * axis.x;
    vec_n[(0, 1)] = axis.x * axis.y;
    vec_n[(0, 2)] = axis.x * axis.z;
    vec_n[(0, 3)] = 0.0;

    vec_n[(1, 0)] = axis.x * axis.y;
    vec_n[(1, 1)] = axis.y * axis.y;
    vec_n[(1, 2)] = axis.y * axis.z;
    vec_n[(1, 3)] = 0.0;

    vec_n[(2, 0)] = axis.z * axis.x;
    vec_n[(2, 1)] = axis.z * axis.y;
    vec_n[(2, 2)] = axis.z * axis.z;
    vec_n[(2, 3)] = 0.0;

    vec_n[(3, 0)] = 0.0;
    vec_n[(3, 1)] = 0.0;
    vec_n[(3, 2)] = 0.0;
    vec_n[(3, 3)] = 1.0;

    let mut sub:Matrix4<f64> = Matrix4::identity();

    sub[(0, 0)] = 0.0;
    sub[(0, 1)] = -axis.z;
    sub[(0, 2)] = axis.y;
    sub[(0, 3)] = 0.0;

    sub[(1, 0)] = axis.z;
    sub[(1, 1)] = 0.0;
    sub[(1, 2)] = -axis.x;
    sub[(1, 3)] = 0.0;

    sub[(2, 0)] = -axis.y;
    sub[(2, 1)] = axis.x;
    sub[(2, 2)] = 0.0;
    sub[(2, 3)] = 0.0;

    sub[(3, 0)] = 0.0;
    sub[(3, 1)] = 0.0;
    sub[(3, 2)] = 0.0;
    sub[(3, 3)] = 0.0;

    model = model * angle.to_radians().cos() + vec_n * (1.0 - angle.to_radians().cos()) + sub * angle.to_radians().sin();

    model
}
pub(crate) fn frame_buffer2cv_mat(frame_buffer: &Vec<V3d>) -> opencv::core::Mat {
    let mut image = unsafe {
        Mat::new_rows_cols_with_data(
            700, 700,
            opencv::core::CV_64FC3,
            frame_buffer.as_ptr() as *mut c_void,
            opencv::core::Mat_AUTO_STEP,
        ).unwrap()
    };
    let mut img = Mat::copy(&image).unwrap();
    image.convert_to(&mut img, opencv::core::CV_8UC3, 1.0, 1.0).expect("panic message");
    cvt_color(&img, &mut image, COLOR_RGB2BGR, 0).unwrap();
    image
}