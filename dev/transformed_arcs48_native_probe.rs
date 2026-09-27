//! Retain original analytic arc transform groups and visibility-only controls.
use sketchpad_rs::document::{Document,records};
use std::{env,fs};
fn main(){
 let args:Vec<_>=env::args().collect();let state=&args[1];let b=fs::read(&args[2]).unwrap();let d=Document::from_bytes(&b,None).unwrap();assert_eq!(d.encode().unwrap(),b);let rs=records(&b).unwrap();
 fs::write(format!("/tmp/transformed_arc48_{state}_input.gsp"),&b).unwrap();
 let ids:Vec<usize>=args[3..].iter().map(|x|x.parse().unwrap()).collect();
 for &id in &ids {let o=&d.objects[id-1];let at=o.header_offset.unwrap();let i=rs.iter().position(|r|r.kind==0x7d0&&r.data.start==at).unwrap();let j=(i..rs.len()).find(|&x|rs[x].kind==0x7d7).unwrap();let group=&b[rs[i].data.start-8..rs[j].data.end];fs::write(format!("tests/fixtures/transformed_arc48_{state}_{id}_group.bin"),group).unwrap();println!("{state} id{id} page{} {:?} pos{:?}",o.page,o.kind,d.position(id));for r in &rs[i..=j]{println!("record {:04x} len{} {:?}",r.kind,r.data.len(),&b[r.data.clone()])}}
 if state=="rolling" {
  for (source,center,start,end,vector_end) in [(21,1,19,20,31),(66,46,64,65,76)] {
   let c=d.position(center).unwrap();let s=d.position(start).unwrap();let e=d.position(end).unwrap();let v=d.position(vector_end).unwrap();let radius=c.distance(s);let a=(c.y-s.y).atan2(s.x-c.x);let z=(c.y-e.y).atan2(e.x-c.x);let sweep=(z-a).rem_euclid(std::f64::consts::TAU);
   println!("ANALYTIC source{source} center{c:?} start{s:?} end{e:?} radius{radius:.15} angle{a:.15} sweep{sweep:.15} targetcenter{v:?}");
   for t in [0.,0.25,0.5,0.75,1.] {let angle=a+sweep*t;println!("translated t{t} x{:.15} y{:.15}",v.x+radius*angle.cos(),v.y-radius*angle.sin());}
  }
  for (name,visible) in [("isolated33",vec![33]),("anchors33",vec![1,31,33]),("isolated83",vec![83]),("anchors83",vec![46,76,83])] {
   let mut patch=b.clone();for o in &d.objects {if let Some(at)=o.header_offset {patch[at+2]=if visible.contains(&o.id){patch[at+2]&!1}else{patch[at+2]|1};}}
   let check=Document::from_bytes(&patch,None).unwrap();assert_eq!(check.encode().unwrap(),patch);fs::write(format!("/tmp/transformed_arc48_{name}_input.gsp"),&patch).unwrap();
   if name.ends_with("33") {let mut page=check;page.active_page=0;let encoded=page.encode().unwrap();page.verify_roundtrip().unwrap();fs::write(format!("/tmp/transformed_arc48_{name}_page0_input.gsp"),encoded).unwrap();}
  }
 }
}
