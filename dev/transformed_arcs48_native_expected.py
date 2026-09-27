"""Independent similarity matrices for Native control geometric expectations."""
import math
P=37.79527559055118
origin=(200.,200.)

def translate(p,x,y):return p[0]+x,p[1]+y

def rotate(p,angle):
 a=math.radians(angle);x,y=p[0]-200,p[1]-200
 return 200+x*math.cos(a)+y*math.sin(a),200-x*math.sin(a)+y*math.cos(a)

def dilate(p,ratio):return 200+ratio*(p[0]-200),200+ratio*(p[1]-200)

def reflect(p):return 440-p[0],p[1]

def operation(state):
 if state in ('vector','vector_on_circle','vector_threepoint','vector_threepoint_collinear'):return lambda p:translate(p,60,20)
 if state in ('fixed_cartesian','fixed_horizontal','fixed_vertical','marked_cartesian'):return lambda p:translate(p,P,-2*P)
 if state in ('fixed_polar','marked_polar'):return lambda p:translate(p,P*math.cos(math.pi/6),-P/2)
 if state in ('fixed_rotation','numeric_rotation'):return lambda p:rotate(p,30)
 if state=='point_angle_rotation':return lambda p:rotate(p,-90)
 if state in ('fixed_dilation','numeric_dilation'):return lambda p:dilate(p,-.5)
 if state=='point_ratio_dilation':return lambda p:dilate(p,2)
 if state=='reflection':return reflect
 if state.startswith('chain'):return lambda p:dilate(reflect(translate(p,60,20)),-.5)
 raise ValueError(state)

def samples_for(state):
 if state=='vector_threepoint_collinear':return [(200.,200.),(120.,200.)]
 if state=='vector_threepoint':
  radius=math.sqrt(800)
  return [(180+radius*math.cos(math.pi/4+math.pi*t),180+radius*math.sin(math.pi/4+math.pi*t)) for t in [0,.25,.5,.75,1]]
 fn=operation(state);radius=60 if state.endswith('_edited') else 40
 return [fn((100+radius*math.cos(math.pi*t/2),180-radius*math.sin(math.pi*t/2))) for t in [0,.25,.5,.75,1]]

if __name__=='__main__':
 states=['vector','fixed_cartesian','fixed_horizontal','fixed_vertical','marked_cartesian','fixed_polar','marked_polar','fixed_rotation','numeric_rotation','point_angle_rotation','fixed_dilation','numeric_dilation','point_ratio_dilation','reflection','chain','chain_edited']
 for state in states:
  fn=operation(state);radius=60 if state.endswith('_edited') else 40
  center=fn((100.,180.));samples=[fn((100+radius*math.cos(math.pi*t/2),180-radius*math.sin(math.pi*t/2))) for t in [0,.25,.5,.75,1]]
  measured_radius=math.dist(center,samples[0])
  print(state,'center',center,'radius',measured_radius,'samples',samples)
