"""Check independently predicted analytic arc samples against Native pixels."""
import importlib.util, math, subprocess, sys
from pathlib import Path
spec=importlib.util.spec_from_file_location('expect',Path(__file__).with_name('transformed_arcs48_native_expected.py'))
expect=importlib.util.module_from_spec(spec);spec.loader.exec_module(expect)
fixture=Path('tests/fixtures')
for path in sorted(fixture.glob('transformed_arc48_api_*_native_verified.png')):
 state=path.name.removeprefix('transformed_arc48_api_').removesuffix('_native_verified.png')
 if len(sys.argv)>1 and state not in sys.argv[1:]:continue
 result=subprocess.check_output(['/nix/store/i22x77kkpiss54gc4xrddx5kv7bk8inf-imagemagick-7.1.2-31/bin/magick',str(path),'-crop','396x250+59+69','+repage','txt:-'],text=True)
 green=[];red=[]
 for line in result.splitlines()[1:]:
  xy,rest=line.split(':',1);x,y=map(int,xy.split(','));x+=59;y+=69;channels=rest.split('(',1)[1].split(')',1)[0].split(',');r,g,b=map(float,channels[:3])
  if 59<=x<=454 and 69<=y<=318:
   if r<70 and g>70 and b<70:green.append((x,y))
   if r>200 and g<80 and b<80:red.append((x,y))
 if state.startswith('numeric_dilation_zero'):print(state,'greenPixelCount',len(green),'redPixelCount',len(red));continue
 fn=expect.operation(state);radius=60 if state.endswith('_edited') else 40
 samples=expect.samples_for(state)
 distances=[min((math.dist((p[0]+59,p[1]+69),q) for q in green),default=math.inf) for p in samples]
 point=samples[1];red_distance=min((math.dist((point[0]+59,point[1]+69),q) for q in red),default=math.inf) if state!='vector_threepoint_collinear' else None
 bounds=(min(x for x,y in green),max(x for x,y in green),min(y for x,y in green),max(y for x,y in green)) if green else None
 print(state,'greenBounds',bounds,'sampleDistances',distances,'constrainedPointDistance',red_distance)
