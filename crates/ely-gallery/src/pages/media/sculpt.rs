use std::f32::consts::TAU;

use ely_gpui_component::media::Orbit;
use image::{Frame, RgbaImage};

type Vec3 = [f32; 3];

/// The ring's radius, its tube's, and the steps around each.
const RING: f32 = 1.0;
const TUBE: f32 = 0.38;
const AROUND: usize = 56;
const ACROSS: usize = 24;
/// Half the camera's field of view, and how far it sits when the torus just fits.
const HALF_VIEW: f32 = 0.35;
const FIT: f32 = 1.55 / 0.35;
/// A matte stone, lit from the upper left.
const STONE: Vec3 = [196.0, 186.0, 172.0];

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: Vec3, b: Vec3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn unit(a: Vec3) -> Vec3 {
    let length = dot(a, a).sqrt();
    [a[0] / length, a[1] / length, a[2] / length]
}

/// A point on the torus, `u` around the ring and `v` around the tube.
fn point(u: f32, v: f32) -> Vec3 {
    let reach = RING + TUBE * v.cos();
    [reach * u.cos(), TUBE * v.sin(), reach * u.sin()]
}

/// The torus as seen from `orbit`, `width` by `height` pixels in BGRA on a clear ground.
pub fn draw(orbit: Orbit, width: u32, height: u32) -> Frame {
    let away = FIT * orbit.distance;
    let eye = [
        away * orbit.pitch.cos() * orbit.yaw.sin(),
        away * orbit.pitch.sin(),
        away * orbit.pitch.cos() * orbit.yaw.cos(),
    ];
    let ahead = unit(sub([0.0; 3], eye));
    let right = unit(cross(ahead, [0.0, 1.0, 0.0]));
    let up = cross(right, ahead);
    let light = unit([
        -right[0] + 1.6 * up[0] - ahead[0],
        -right[1] + 1.6 * up[1] - ahead[1],
        -right[2] + 1.6 * up[2] - ahead[2],
    ]);
    let focal = height as f32 / 2.0 / HALF_VIEW.tan();
    let project = |p: Vec3| {
        let seen = sub(p, eye);
        let depth = dot(seen, ahead);
        (
            width as f32 / 2.0 + dot(seen, right) / depth * focal,
            height as f32 / 2.0 - dot(seen, up) / depth * focal,
            depth,
        )
    };
    let step = |ix: usize, count: usize| ix as f32 / count as f32 * TAU;
    let mut faces = Vec::with_capacity(AROUND * ACROSS);
    for i in 0..AROUND {
        for j in 0..ACROSS {
            let corners = [
                point(step(i, AROUND), step(j, ACROSS)),
                point(step(i + 1, AROUND), step(j, ACROSS)),
                point(step(i + 1, AROUND), step(j + 1, ACROSS)),
                point(step(i, AROUND), step(j + 1, ACROSS)),
            ];
            let normal = unit(cross(
                sub(corners[3], corners[0]),
                sub(corners[1], corners[0]),
            ));
            if dot(normal, sub(eye, corners[0])) <= 0.0 {
                continue;
            }
            let lit = 0.42 + 0.58 * dot(normal, light).max(0.0);
            let shade = STONE.map(|channel| (channel * lit).min(255.0) as u8);
            let placed = corners.map(project);
            let depth = placed.iter().map(|corner| corner.2).sum::<f32>();
            faces.push((depth, placed.map(|(x, y, _)| (x, y)), shade));
        }
    }
    faces.sort_by(|a, b| b.0.total_cmp(&a.0));
    let mut pixels = RgbaImage::new(width, height);
    for (_, [a, b, c, d], [red, green, blue]) in faces {
        for triangle in [[a, b, c], [a, c, d]] {
            fill(&mut pixels, triangle, [blue, green, red, 255]);
        }
    }
    Frame::new(pixels)
}

/// Fills a triangle given either way round.
fn fill(pixels: &mut RgbaImage, [a, b, c]: [(f32, f32); 3], color: [u8; 4]) {
    let edge = |p: (f32, f32), q: (f32, f32), x: f32, y: f32| {
        (q.0 - p.0) * (y - p.1) - (q.1 - p.1) * (x - p.0)
    };
    let area = edge(a, b, c.0, c.1);
    if area.abs() < f32::EPSILON {
        return;
    }
    let (width, height) = (pixels.width() as f32, pixels.height() as f32);
    let low_x = a.0.min(b.0).min(c.0).max(0.0) as u32;
    let high_x = a.0.max(b.0).max(c.0).min(width - 1.0).max(0.0) as u32;
    let low_y = a.1.min(b.1).min(c.1).max(0.0) as u32;
    let high_y = a.1.max(b.1).max(c.1).min(height - 1.0).max(0.0) as u32;
    for y in low_y..=high_y {
        for x in low_x..=high_x {
            let (x_at, y_at) = (x as f32 + 0.5, y as f32 + 0.5);
            let inside = [
                edge(a, b, x_at, y_at),
                edge(b, c, x_at, y_at),
                edge(c, a, x_at, y_at),
            ]
            .iter()
            .all(|side| side * area >= 0.0);
            if inside {
                pixels.put_pixel(x, y, image::Rgba(color));
            }
        }
    }
}
