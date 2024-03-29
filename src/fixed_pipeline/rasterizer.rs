
use crate::{math::{matrix::Mat4x4f, vector::{Vector4f, Vector3f}}, common::texture::Texture};
use crate::common::triangle::Triangle;
use crate::common::light::Light;

use super::{edge_walking::draw_trangle_edge_walking, edge_equation::{draw_trangle_edge_equation, draw_trangle_edge_equation_result}};

pub struct RenderResult {
    pub idx: i32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub z: f32
}

impl RenderResult {
    pub fn new() -> RenderResult {
        RenderResult { idx: 0, r: 0, g: 0, b: 0, z: 0.0 }
    }
}

pub struct Rasterizer {
    model: Mat4x4f,
    view: Mat4x4f,
    projection: Mat4x4f,
    mvp: Mat4x4f,
    mv: Mat4x4f,
    lights: Vec<Light>,
    eye_pos: Vector3f
}

impl Rasterizer {
    pub fn new() -> Rasterizer {
        Rasterizer {
            model: Mat4x4f::identity(),
            view: Mat4x4f::identity(),
            mv: Mat4x4f::identity(),
            projection: Mat4x4f::identity(),
            mvp: Mat4x4f::identity(),
            lights: Vec::new(),
            eye_pos: Vector3f::new()
        }
    }

    pub fn set_view(&mut self, m: Mat4x4f) {
        self.view = m;
    }

    pub fn set_model(&mut self, m: Mat4x4f) {
        self.model = m;
    }

    pub fn set_eye_pos(&mut self, v: Vector3f) {
        self.eye_pos = v;
    }

    pub fn set_lights(&mut self, lights: Vec<Light>) {
        self.lights = lights;
    }

    pub fn set_projection(&mut self, m: Mat4x4f) {
        self.projection = m;
    }

    pub fn compute_mvp(&mut self) {
        self.mvp = self.projection.mul(&self.view).mul(&self.model);
        self.mv = self.view.mul(&self.model);
    }

    pub fn get_lights(&self) -> &Vec<Light> {
        &self.lights
    }

    pub fn get_eye_pos(&self) -> &Vector3f {
        &self.eye_pos
    }
}

pub fn draw_trangle_map(rasterizer: &Rasterizer, 
    width: i32, 
    height: i32, 
    triangle: &mut Triangle,
    textures: &Vec<Texture>
) -> Vec<RenderResult> {
    let t1 = rasterizer.mvp.apply(&triangle.vertexs[0].origin_v);
    let t2 = rasterizer.mvp.apply(&triangle.vertexs[1].origin_v);
    let t3 = rasterizer.mvp.apply(&triangle.vertexs[2].origin_v);

    triangle.set_tvetexs(vec![
        rasterizer.mv.apply(&triangle.vertexs[0].origin_v),
        rasterizer.mv.apply(&triangle.vertexs[1].origin_v),
        rasterizer.mv.apply(&triangle.vertexs[2].origin_v),
        ]);

    let view_port = get_view_port(width as f32, height as f32);
    let mut p1 = view_port.apply(&t1);
    let mut p2 = view_port.apply(&t2);
    let mut p3 = view_port.apply(&t3);

    triangle.set_vertexs(vec![p1, p2, p3]);
    return draw_trangle_edge_equation_result(rasterizer, width, height, triangle, textures);
}

pub fn draw_trangle(rasterizer: &Rasterizer, 
    image: &mut Vec<u8>, 
    zbuf: &mut Vec<f32>,
    near: f32,
    far: f32,
    width: i32, 
    height: i32, 
    triangle: &mut Triangle,
    textures: &Vec<Texture>
) {
    let t1 = rasterizer.mvp.apply(&triangle.vertexs[0].origin_v);
    let t2 = rasterizer.mvp.apply(&triangle.vertexs[1].origin_v);
    let t3 = rasterizer.mvp.apply(&triangle.vertexs[2].origin_v);

    triangle.set_tvetexs(vec![
        rasterizer.mv.apply(&triangle.vertexs[0].origin_v),
        rasterizer.mv.apply(&triangle.vertexs[1].origin_v),
        rasterizer.mv.apply(&triangle.vertexs[2].origin_v),
        ]);

    let view_port = get_view_port(width as f32, height as f32);
    let mut p1 = view_port.apply(&t1);
    let mut p2 = view_port.apply(&t2);
    let mut p3 = view_port.apply(&t3);

    triangle.set_vertexs(vec![p1, p2, p3]);

    //draw_trangle_edge_walking(image, rasterizer, zbuf, width, height, &triangle, textures);
    draw_trangle_edge_equation(image, rasterizer, zbuf, width, height, triangle, textures);
}

pub fn get_view_matrix(eye: Vector4f, at: Vector4f, mut up: Vector4f) -> Mat4x4f {
    let mut g = at.sub(&eye);
    g.normlize();
    up.normlize();
    let mut x = g.cross_product(&up);
    x.normlize();
    let m = vec![
        vec![x.x(), x.y(), x.z(), -eye.x()],
        vec![up.x(), up.y(), up.z(), -eye.y()],
        vec![-g.x(), -g.y(), -g.z(), -eye.z()],
        vec![0.0, 0.0, 0.0, 1.0]];
    Mat4x4f::new_val(m)
}

pub fn get_view_port(width: f32, height: f32) -> Mat4x4f {
    let m = vec![
            vec![width/2.0, 0.0, 0.0, width/2.0],
            vec![0.0, -height/2.0, 0.0, height/2.0],
            vec![0.0, 0.0, 1.0, 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ];
    
    Mat4x4f::new_val(m)
}

pub fn get_model_matrix(angel: f32) -> Mat4x4f {
    let r = std::f32::consts::PI * angel / 180.0;
    let m = vec![
            vec![r.cos(), 0.0, r.sin(), 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            vec![-r.sin(), 0.0, r.cos(), 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ];
    Mat4x4f::new_val(m)
}

pub fn get_ortho_projection_matrix(l: f32, r: f32, t: f32, b: f32, n: f32, f: f32) -> Mat4x4f {
    //映射z到(-1,0)
    let m1 = Mat4x4f::new_val( 
        vec![
            vec![2.0/(r - l), 0.0, 0.0, 0.0],
            vec![0.0, 2.0/(t - b), 0.0, 0.0],
            vec![0.0, 0.0, 1.0/(n - f), 0.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ]);
    let m2 = Mat4x4f::new_val(
        vec![
            vec![1.0, 0.0, 0.0, -(l+r)/2.0],
            vec![0.0, 1.0, 0.0, -(t+b)/2.0],
            vec![0.0, 0.0, 1.0, -(n+f+1.0)/2.0],
            vec![0.0, 0.0, 0.0, 1.0],
        ]);
    m1.mul(&m2)
}

pub fn get_presp_projection_matrix(eye_fov: f32, aspect_ratio: f32, near: f32, far: f32) -> Mat4x4f {
    let angle = eye_fov * std::f32::consts::PI / 180.0;
    //let height = near * angle.tan();
    //let width = height * aspect_ratio;

    let t = near.abs() * (angle/2.0).tan();
    let r = t * aspect_ratio;
    let l = -r;
    let b = -t;

    get_ortho_projection_matrix(l, r, t, b, near, far).mul(&Mat4x4f::new_val(
        vec![
            vec![near, 0.0, 0.0, 0.0],
            vec![0.0, near, 0.0, 0.0],
            vec![0.0, 0.0, near+far, -near*far],
            vec![0.0, 0.0, 1.0, 0.0],
        ]))
}
