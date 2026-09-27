//! Actual public API controls for all analytic arc transform wire flavors.
use sketchpad_rs::document::{Document,Kind,Point};
use sketchpad_rs::document::transform_model::{ElementType,TransformDefinition,Transform,DisplacementExpr,NumericExpr,NativeTransformFlavor};
use sketchpad_rs::table_records::NumericUnit;
use std::{env,fs};
fn literal(value:f64,unit:NumericUnit)->NumericExpr{NumericExpr::Literal{magnitude:value,unit,scale:if unit==NumericUnit::Length{37.79527559055118}else{1.}}}
fn object(id:usize,unit:NumericUnit)->NumericExpr{NumericExpr::ObjectValue{id,unit}}
fn main(){
 let state=env::args().nth(1).unwrap();
 if state=="vector_on_circle"{on_circle();return;}
 let mut d=Document::new();
 for (x,y) in [(100.,180.),(140.,180.),(100.,140.),(200.,200.),(80.,80.),(140.,100.),(220.,80.),(220.,300.)]{d.add(Kind::Free(Point{x,y}));}
 d.add(if state=="vector_on_circle"{Kind::Circle(1,2)}else{Kind::Segment(7,8)});
 for (x,y) in [(200.,160.),(240.,200.),(280.,200.)]{d.add(Kind::Free(Point{x,y}));}
 d.add(if state.starts_with("vector_threepoint"){Kind::ThreePointArc(2,1,3)}else if state=="vector_on_circle"{Kind::ArcOnCircle(9,2,3)}else{Kind::CenterArc(1,2,3)});
 for expr in ["1 cm","2 cm","30°"]{d.add_calculation_expression(expr,(10,10)).unwrap();}
 d.add_named_parameter("q",-0.5,(10,10)).unwrap();
 use NativeTransformFlavor as F;use Transform as T;use DisplacementExpr as D;
 let (transform,flavor)=match state.as_str(){
 "vector"|"vector_threepoint"|"vector_threepoint_collinear"|"vector_on_circle"|"chain"|"chain_edited"=>(T::Translate(D::Vector{start:5,end:6}),F::Vector),
 "fixed_cartesian"=>(T::Translate(D::Cartesian{horizontal:literal(1.,NumericUnit::Length),vertical:literal(2.,NumericUnit::Length)}),F::FixedCartesian),
 "fixed_horizontal"=>(T::Translate(D::Cartesian{horizontal:literal(1.,NumericUnit::Length),vertical:object(15,NumericUnit::Length)}),F::FixedHorizontal),
 "fixed_vertical"=>(T::Translate(D::Cartesian{horizontal:object(14,NumericUnit::Length),vertical:literal(2.,NumericUnit::Length)}),F::FixedVertical),
 "marked_cartesian"=>(T::Translate(D::Cartesian{horizontal:object(14,NumericUnit::Length),vertical:object(15,NumericUnit::Length)}),F::MarkedCartesian),
 "fixed_polar"=>(T::Translate(D::Polar{angle:literal(30.,NumericUnit::Degrees),distance:literal(1.,NumericUnit::Length)}),F::FixedPolar),
 "marked_polar"=>(T::Translate(D::Polar{angle:literal(30.,NumericUnit::Degrees),distance:object(14,NumericUnit::Length)}),F::MarkedPolar),
 "fixed_rotation"=>(T::Rotate{center:4,angle:literal(30.,NumericUnit::Degrees)},F::FixedRotation),
 "numeric_rotation"=>(T::Rotate{center:4,angle:object(16,NumericUnit::Degrees)},F::NumericRotation),
 "point_angle_rotation"=>(T::Rotate{center:4,angle:NumericExpr::DirectedAngle{first:10,vertex:4,last:11}},F::PointAngleRotation),
 "fixed_dilation"=>(T::Dilate{center:4,ratio:literal(-0.5,NumericUnit::Scalar)},F::FixedDilation),
 "numeric_dilation"|"numeric_dilation_zero"=>(T::Dilate{center:4,ratio:object(17,NumericUnit::Scalar)},F::NumericDilation),
 "point_ratio_dilation"=>(T::Dilate{center:4,ratio:NumericExpr::DistanceRatio{origin:4,base:11,target:12}},F::PointRatioDilation),
 "reflection"=>(T::Reflect{mirror:9},F::Reflection),
 _=>panic!("unknown state")};
 let id=d.add(Kind::Transformed(TransformDefinition::new(13,ElementType::Arc,transform,flavor).unwrap()));assert_eq!(id,18);
 let final_id=if state.starts_with("chain") {d.add(Kind::Transformed(TransformDefinition::new(18,ElementType::Arc,T::Reflect{mirror:9},F::Reflection).unwrap()));d.add(Kind::Transformed(TransformDefinition::new(19,ElementType::Arc,T::Dilate{center:4,ratio:literal(-0.5,NumericUnit::Scalar)},F::FixedDilation).unwrap()))}else{18};
 let (point,angle,length)=if state=="numeric_dilation_zero"||state=="vector_threepoint_collinear"{(0,0,0)}else{(d.add(Kind::PointOnArc(final_id,0.25)),d.add(Kind::ArcAngle(final_id)),d.add(Kind::ArcLength(final_id)))};
 if state=="chain_edited"{assert!(d.move_point(2,Point{x:160.,y:180.}));}
 if state=="vector_threepoint_collinear"{assert!(d.move_point(3,Point{x:60.,y:180.}));}
 for o in &mut d.objects{o.visible=[final_id,point].contains(&o.id);}
 d.verify_roundtrip().unwrap();let baseline=d.encode().unwrap();d=Document::from_bytes(&baseline,None).unwrap();assert_eq!(d.encode().unwrap(),baseline);
 if state=="numeric_dilation_zero"{d.set_parameter_value(17,0.).unwrap();}
 d.verify_roundtrip().unwrap();let b=d.encode().unwrap();let r=Document::from_bytes(&b,None).unwrap();assert_eq!(r.encode().unwrap(),b);
 println!("{state} source13 final{final_id} point{point} angle{angle} length{length} pointPosition{:?} arcAngle{:?} arcLength{:?}",d.position(point),d.arc_angle_degrees(angle),d.arc_length_cm(length));
 if let Some(points)=d.arc_points(final_id,4){for (i,p) in points.iter().enumerate(){println!("sample{i} {:.15} {:.15}",p.x,p.y);}}else{println!("undefined analytic arc");}
 fs::write(format!("/tmp/transformed_arc48_api_{state}_input.gsp"),b).unwrap();
}

fn on_circle(){
 let mut d=Document::new();d.add(Kind::Free(Point{x:100.,y:180.}));d.add(Kind::Free(Point{x:140.,y:180.}));d.add(Kind::Circle(1,2));d.add(Kind::PointOnCircle(3,1.,0.));d.add(Kind::PointOnCircle(3,0.,1.));d.add(Kind::ArcOnCircle(3,4,5));d.add(Kind::Free(Point{x:80.,y:80.}));d.add(Kind::Free(Point{x:140.,y:100.}));
 d.add(Kind::Transformed(TransformDefinition::new(6,ElementType::Arc,Transform::Translate(DisplacementExpr::Vector{start:7,end:8}),NativeTransformFlavor::Vector).unwrap()));
 d.add(Kind::PointOnArc(9,0.25));d.add(Kind::ArcAngle(9));d.add(Kind::ArcLength(9));
 for o in &mut d.objects{o.visible=[9,10].contains(&o.id);}
 d.verify_roundtrip().unwrap();let b=d.encode().unwrap();assert_eq!(Document::from_bytes(&b,None).unwrap().encode().unwrap(),b);
 println!("vector_on_circle source6 circle3 start4 end5 final9 point10 angle11 length12 pointPosition{:?}",d.position(10));for(i,p)in d.arc_points(9,4).unwrap().iter().enumerate(){println!("sample{i} {:.15} {:.15}",p.x,p.y);}
 fs::write("/tmp/transformed_arc48_api_vector_on_circle_input.gsp",b).unwrap();
}
