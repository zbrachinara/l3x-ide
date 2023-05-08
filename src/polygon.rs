use std::{f32::consts::PI};

//Current shapes package can't do arbitrary polygons?
use macroquad::prelude::*;
//Doesn't handle the non-convex case properly -- I need circular linked lists!
/*fn triangulate_helper(root: Vec2, pts: &[Vec2]) -> Vec<[Vec2; 3]> {
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
}*/
fn positive_angle(u: Vec2, v: Vec2) -> f32{
    let res=u.angle_between(v);
    if res>=0. {res} else {2.*PI+res}
}
pub fn triangulate(pts: Vec<Vec2>) -> Vec<[Vec2;3]> {
    if pts.len()==3 {
        return vec!([pts[0], pts[1], pts[2]]);
    }
    let mut diagonal_stack=vec![(1,0.)];
    let mut angle = 0.;
    for i in 2..pts.len() {
        let leadingAngle=diagonal_stack.last().unwrap().1;
        let visible = angle==leadingAngle;
        angle+=(pts[i-1]-pts[0]).angle_between(pts[i]-pts[0]);
        //println!("{}", angle);
        if visible {
            if positive_angle(pts[0]-pts[i-1],pts[i]-pts[i-1])<positive_angle(pts[0]-pts[i-1], pts[i-2]-pts[i-1]){
                //cuts previous diagonal
                diagonal_stack.pop();
                while angle<diagonal_stack.last().unwrap().1 && (pts[i]-pts[i-1]).angle_between(pts[diagonal_stack.last().unwrap().0]-pts[i-1])>0. {
                    diagonal_stack.pop();
                }
            }
        }
        let leadingAngle=diagonal_stack.last().unwrap().1;
        if angle>leadingAngle {
            diagonal_stack.push((i, angle));
        }
    }
    /*for t in &diagonal_stack {
        println!("{}", t.0);
    }*/
    let mut products: Vec<Vec<Vec2>>=vec![];
    for (i, (index, _)) in diagonal_stack[..diagonal_stack.len()-1].iter().enumerate(){
        let mut res = Vec::from_iter(pts[*index..diagonal_stack[i+1].0+1].iter().cloned());
        res.push(pts[0]);
        products.push(res);
    }
    /*for l in &products {
        for v in l {
            println!("{}", v);
        }
        println!();
    }*/
    products.into_iter().map(|p| triangulate(p)).flatten().collect()
}
pub fn draw_polygon(pts: Vec<Vec2>, color: Color){
    draw_triangulation(triangulate(pts), color)
}
pub fn draw_triangulation(triangles: Vec<[Vec2; 3]>, color: Color) {
    for [a,b,c] in triangles {
        draw_triangle(a,b,c,color);
    }
}
#[test]
fn test(){
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
    for p in out_arrow_triangles{
        for q in p {
            println!("{}", q);
        }
        println!();
    }
}