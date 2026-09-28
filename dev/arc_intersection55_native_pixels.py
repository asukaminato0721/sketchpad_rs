"""Independent circle roots, ordered straight branches and actual Native points."""
from pathlib import Path
import math,subprocess
M='/nix/store/i22x77kkpiss54gc4xrddx5kv7bk8inf-imagemagick-7.1.2-31/bin/magick'
S=['center_both','center_quarter','line_reverse','ray_domain','segment_domain','endpoints','tangent','nointersection','three_circular','three_linear','arc_translated','arc_reflected','arc_major']
for state in S:
 path=Path('tests/fixtures/arc_intersection55_'+state+'_native_saved.png')
 if not path.exists():continue
 w,h=map(int,subprocess.check_output([M,'identify','-format','%w %h',str(path)],text=True).split());b=subprocess.check_output([M,str(path),'-depth','8','rgb:-'])
 points={c:[] for c in [(255,0,0),(0,0,255)]}
 for i in range(w*h):
  if i%w<60 or i//w<180:continue
  c=tuple(b[3*i:3*i+3])
  if c in points:points[c].append((i%w,i//w))
 y=200 if state=='endpoints' else 140 if state=='tangent' else 120 if state=='nointersection' else 170
 cx=200+37.79527559055118 if state=='arc_translated' else 200
 delta=math.sqrt(max(0,3600-(y-200)**2))
 red=(cx-delta,y);blue=(cx+delta,y)
 expected={ (255,0,0):red, (0,0,255):blue }
 occluded=state in ['center_quarter','line_reverse','ray_domain','tangent','arc_reflected','arc_major']
 if state in ['arc_reflected','arc_major']:expected[(0,0,255)]=(cx-delta,y)
 if state in ['segment_domain','nointersection']:expected={c:None for c in points}
 if state=='three_linear':
  print(state,'UNRESOLVED',[(c,len(ps),min(ps) if ps else None,max(ps) if ps else None) for c,ps in points.items()],flush=True);continue
 for color,p in expected.items():
  if p is None:
   print(state,color,'undefined count',len(points[color]),flush=True);assert not points[color]
  elif occluded and color==(255,0,0):
   print(state,'occludedred count',len(points[color]),'definedness requires independentcaption',flush=True)
  else:
   e=(p[0]+59,p[1]+51);d=min((math.dist(e,q) for q in points[color]),default=math.inf)
   print(state,color,'target',p,'sampleDistance',d,flush=True);assert d<=1.5
