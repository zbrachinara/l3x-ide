use std::ops::Index;

use const_push::ConstVec;
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
        &self.slice[self.indices[index] as usize]
    }
}

const fn const_triangulate_indices_inner<const N: usize>(vertices: &[Vec2], indices: &[u16])-> ConstVec<u16,N> {
    assert!(
        indices.len() >= 3,
        "Triangulation produced a low (<3) number of indices"
    );
    assert!(
        vertices.len() >= 3,
        "Attempted triangulation of something with less than tri vertices"
    );

    // if indices.len() == 3 {
    //     return indices.to_vec();
    // }
    todo!()
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

#[test]
fn test() {
    let arrow_vertices = vec![
        vec2(-0., 0.75),
        vec2(-0.25, 1.0),
        vec2(-0.5, 0.75),
        vec2(-0.3, 0.75),
        vec2(-0.3, 0.25),
        vec2(-0.2, 0.25),
        vec2(-0.2, 0.75),
    ];
    let out_arrow_triangles = triangulate_indices(&arrow_vertices);
    for p in &out_arrow_triangles
        .into_iter()
        .map(|ix| (ix, arrow_vertices[ix as usize]))
        .chunks(3)
    {
        println!("{:?}", p.collect_vec());
    }
}
