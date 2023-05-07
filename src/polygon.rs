use std::f32::INFINITY;

//Current shapes package can't do arbitrary polygons?
use macroquad::prelude::*;
//Doesn't handle the non-convex case properly -- I need circular linked lists!
fn triangulate_helper(root: Vec2, pts: &[Vec2]) -> Vec<[Vec2; 3]> {
    if pts.len()==2 {
        return vec![[root, pts[0],pts[1]]]
    }
    let right = pts[0];
    let left = pts[pts.len()-1];
    let mut closest: Option<&Vec2> = None;
    let mut min_dist = INFINITY;
    let mut index = 0;
    for (i,p) in pts.iter().enumerate() {
        let d = *p-root;
        if d.perp_dot(right-root)>0. && d.perp_dot(root-left)>0. && (*p-right).perp_dot(left-right)<0. {
            let dist=d.dot(d);
            if dist<min_dist {
                closest=Some(p);
                min_dist=dist;
                index=i;
            }
        }
    }
    if closest.is_none(){
        let mut res = triangulate_helper(right,&pts[1..]);
        res.push([root, right, left]);
        return res
    }
    let mut first = triangulate_helper(root, &pts[..index+1]);
    let mut second = triangulate_helper(root, &pts[index..]);
    first.append(&mut second);
    first
}

pub fn triangulate(pts: &[Vec2])->Vec<[Vec2;3]>{
    triangulate_helper(pts[0], &pts[1..])
}
pub fn draw_polygon(pts: &[Vec2], color: Color){
    draw_triangulation(triangulate(pts), color)
}
pub fn draw_triangulation(triangles: Vec<[Vec2; 3]>, color: Color) {
    for [a,b,c] in triangles {
        draw_triangle(a,b,c,color);
    }
}
#[test]
fn test(){
    let shape = [vec2(0.,0.),vec2(1.,0.),vec2(0.5,0.5),vec2(1.,1.),vec2(0.,1.)];
    let res = triangulate(&shape);
    for p in res{
        for q in p {
            println!("{}", q);
        }
        println!();
    }
    println!("{}", vec2(0., 1.).perp_dot(vec2(1.,0.)));
}