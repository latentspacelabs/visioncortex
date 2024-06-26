use std::f64::consts::PI;

use crate::{Point2, PointF64, PointI32};

/// assume origin is top left corner, signed_area > 0 imply clockwise
pub(super) fn signed_area(p1: PointI32, p2: PointI32, p3: PointI32) -> i32 {
    (p2.x - p1.x) * (p3.y - p1.y) - (p3.x - p1.x) * (p2.y - p1.y)
}

/// Given lines p1p2 and p3p4, returns their intersection.
/// If the two lines coincide, returns the mid-pt of p2 and p3.
/// If the two lines are parallel, panicks.
/// https://github.com/tyt2y3/vaser-unity/blob/master/Assets/Vaser/Vec2Ext.cs#L107 (Intersect)
#[inline]
pub(super) fn find_intersection(p1: &PointF64, p2: &PointF64, p3: &PointF64, p4: &PointF64) -> PointF64 {
    try_find_intersection(p1, p2, p3, p4).expect("The two lines are parallel!")
}

pub(super) fn try_find_intersection(p1: &PointF64, p2: &PointF64, p3: &PointF64, p4: &PointF64) -> Option<PointF64> {

    // Notes: for some reason ShapeSense used f64::EPSILON, now it's being reverted to the original value.
    // we might want to make this a parameter if this proves to become a problem.
    const EPSILON: f64 = 1e-7;
    
    let (denom, numera, numerb);
    denom  = (p4.y-p3.y) * (p2.x-p1.x) - (p4.x-p3.x) * (p2.y-p1.y);
    numera = (p4.x-p3.x) * (p1.y-p3.y) - (p4.y-p3.y) * (p1.x-p3.x);
    numerb = (p2.x-p1.x) * (p1.y-p3.y) - (p2.y-p1.y) * (p1.x-p3.x);

    if denom <= EPSILON && numera <= EPSILON && numerb <= EPSILON {
        // The two lines coincide
        return Some(find_mid_point(p2, p3));
    }

    if denom <= EPSILON {
        return None;
    }

    let mua = numera/denom;

    Some(PointF64 { x: p1.x + mua * (p2.x-p1.x), y: p1.y + mua * (p2.y-p1.y) })
}

pub(super) fn find_mid_point(p1: &PointF64, p2: &PointF64) -> PointF64 {
    let x = (p1.x + p2.x) / 2.0;
    let y = (p1.y + p2.y) / 2.0;
    PointF64 {x, y}
}

pub(super) fn norm<T>(p: &Point2<T>) -> f64
where T: std::ops::Add<Output = T> + std::ops::Mul<Output = T> + Copy + Into<f64> {
    let n: f64 = (p.x*p.x + p.y*p.y).into();
    n.sqrt()
}

pub(super) fn normalize<T>(p: &Point2<T>) -> PointF64
where T: std::ops::Add<Output = T> + std::ops::Mul<Output = T> + Copy + Into<f64> {
    let norm = norm(p);
    let (px, py): (f64, f64) = (p.x.into(), p.y.into());
    PointF64::new(px / norm, py / norm)
}

pub(super) fn angle(p: &PointF64) -> f64 {
    if p.y.is_sign_negative() {
        -p.x.acos()
    } else {
        p.x.acos()
    }
}

/// Given angles in (-pi,pi], find the signed angle difference
/// Positive in clockwise direction, 0-degree axis is the positive x axis
pub(super) fn signed_angle_difference(from: &f64, to: &f64) -> f64 {
    let v1 = *from;
    let mut v2 = *to;
    if v1 > v2 {
        v2 += 2.0 * PI;
    }

    let diff = v2 - v1;
    if diff > PI {
        diff - 2.0 * PI
    } else {
        diff
    }
}