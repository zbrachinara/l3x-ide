use std::ops::Index;

use itertools::Itertools;
use macroquad::prelude::*;

fn positive_angle(u: Vec2, v: Vec2) -> f32 {
    u.angle_between(v).rem_euclid(std::f32::consts::TAU)
}

#[derive(Clone, Copy)]
struct IndexedSlice<'a, T> {
    slice: &'a [T],
    indices: &'a [u16],
}

impl<'a, T> IndexedSlice<'a, T> {
    fn get_with_index(&self, ix: usize) -> Option<(u16, &T)> {
        self.indices
            .get(ix)
            .and_then(|ix| self.slice.get(*ix as usize).map(|elem| (*ix, elem)))
    }

    fn get(&self, ix: usize) -> Option<&T> {
        self.get_with_index(ix).map(|(_, e)| e)
    }

    fn iter(&self) -> impl Iterator<Item = (u16, &'a T)> {
        self.indices
            .iter()
            .map(|&ix| (ix, &self.slice[ix as usize]))
    }

    fn iter_elems(&self) -> impl Iterator<Item = &'a T> {
        self.iter().map(|(_, e)| e)
    }
}

impl<'a, T> Index<usize> for IndexedSlice<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).unwrap()
    }
}

pub fn triangulate_indices(vertices: &[Vec2]) -> Vec<u16> {
    log::trace!("doing an index-based triangulation");
    triangulate_indices_inner(
        vertices,
        (0..(vertices.len() as u16)).collect_vec().as_slice(),
    )
}

fn triangulate_indices_inner(vertices: &[Vec2], indices: &[u16]) -> Vec<u16> {
    log::trace!("running an iteration of index-based trinagulation");
    assert!(
        indices.len() >= 3,
        "Triangulation produced a low (<3) number of indices: {indices:?}"
    );
    assert!(
        vertices.len() >= 3,
        "Attempted triangulation of something with less than tri vertices"
    );

    if indices.len() == 3 {
        return indices.to_vec();
    }

    let indexed_vertices = IndexedSlice {
        slice: vertices,
        indices,
    };

    // vec between vertex indices (?) and the angle they make with the line created by the focus
    // vertex and the vertex which comes after it
    let mut diagonal_stack = vec![(1, 0.)];

    let mut angle = 0.;
    let mut prior_was_visible = true;
    let focus = indexed_vertices[0];
    for (ix, (&name_later, &prior, &current)) in
        indexed_vertices.iter_elems().tuple_windows().enumerate()
    {
        angle += (prior - focus).angle_between(current - focus);
        if prior_was_visible
            && positive_angle(focus - prior, current - prior)
                < positive_angle(focus - prior, name_later - prior)
        {
            // cull occluded diagonals
            diagonal_stack.pop();
            while angle < diagonal_stack.last().unwrap().1
                && (current - prior)
                    .angle_between(indexed_vertices[diagonal_stack.last().unwrap().0] - prior)
                    .is_sign_positive()
            {
                diagonal_stack.pop();
            }
        }
        let leading_angle = diagonal_stack.last().unwrap().1;
        if angle > leading_angle {
            diagonal_stack.push((ix + 2, angle));
            prior_was_visible = true;
        } else {
            prior_was_visible = false;
        }
    }

    log::trace!(
        "diagonal stack contains vertices: {:?}",
        diagonal_stack.iter().map(|(a, _)| a).collect_vec()
    );
    diagonal_stack
        .iter()
        .tuple_windows()
        .map(|(&(ix, _), &(jx, _))| {
            let mut res = indices[ix..=jx].to_vec();
            res.push(indices[0]);
            log::trace!("found subpolygon: {res:?}");
            res
        })
        .flat_map(|indices| triangulate_indices_inner(vertices, indices.as_slice()))
        .collect()
}

pub fn triangulate(pts: Vec<Vec2>) -> Vec<[Vec2; 3]> {
    assert!(
        pts.len() >= 3,
        "Attempted triangulation of something with less than tri vertices"
    );
    if pts.len() == 3 {
        return vec![[pts[0], pts[1], pts[2]]];
    }

    // vec between vertex indices (?) and the angle they make with the line created by the focus
    // vertex and the vertex which comes after it
    let mut diagonal_stack = vec![(1, 0.)];

    let mut angle = 0.;
    let mut prior_was_visible = true;
    let focus = pts[0];
    for (ix, (&name_later, &prior, &current)) in pts.iter().tuple_windows().enumerate() {
        angle += (prior - focus).angle_between(current - focus);
        if prior_was_visible
            && positive_angle(focus - prior, current - prior)
                < positive_angle(focus - prior, name_later - prior)
        {
            // cull occluded diagonals
            diagonal_stack.pop();
            while angle < diagonal_stack.last().unwrap().1
                && (current - prior)
                    .angle_between(pts[diagonal_stack.last().unwrap().0] - prior)
                    .is_sign_positive()
            {
                diagonal_stack.pop();
            }
        }
        let leading_angle = diagonal_stack.last().unwrap().1;
        if angle > leading_angle {
            diagonal_stack.push((ix + 2, angle));
            prior_was_visible = true;
        } else {
            prior_was_visible = false;
        }
    }
    log::trace!(
        "diagonal stack contains vertices: {:?}",
        diagonal_stack.iter().map(|(a, _)| a)
    );
    let mut products: Vec<Vec<Vec2>> = vec![];
    for (i, (index, _)) in diagonal_stack[..diagonal_stack.len() - 1]
        .iter()
        .enumerate()
    {
        let mut res = Vec::from_iter(pts[*index..diagonal_stack[i + 1].0 + 1].iter().cloned());
        res.push(pts[0]);
        products.push(res);
    }
    log::trace!("calculated subpolygons: {products:?}");
    products.into_iter().flat_map(triangulate).collect()
}
pub fn draw_polygon(pts: Vec<Vec2>, color: Color) {
    draw_triangulation(triangulate(pts), color)
}
pub fn draw_triangulation(triangles: Vec<[Vec2; 3]>, color: Color) {
    for [a, b, c] in triangles {
        draw_triangle(a, b, c, color);
    }
}
#[test]
fn test() {
    /*let shape = vec![vec2(0.,0.),vec2(1.,0.),vec2(0.5,0.5),vec2(1.,1.),vec2(0.,1.)];
        let res = triangulate(shape);
        for p in res{
            for q in p {
                println!("{}", q);
            }
            println!();
        }
        println!("{}", vec2(0., 1.).perp_dot(vec2(1.,0.)));
    */
    let arrow_vertices = vec![
        vec2(-0., 0.75),
        vec2(-0.25, 1.0),
        vec2(-0.5, 0.75),
        vec2(-0.3, 0.75),
        vec2(-0.3, 0.25),
        vec2(-0.2, 0.25),
        vec2(-0.2, 0.75),
    ];
    let out_arrow_triangles = triangulate(arrow_vertices);
    for p in out_arrow_triangles {
        for q in p {
            println!("{}", q);
        }
        println!();
    }
}
