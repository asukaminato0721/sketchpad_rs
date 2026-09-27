"""Independent Native capture color component bounds; no Rust geometry used."""
from pathlib import Path
import subprocess,re,sys
magick='/nix/store/i22x77kkpiss54gc4xrddx5kv7bk8inf-imagemagick-7.1.2-31/bin/magick'
for state in sys.argv[1:]:
 path=Path('/tmp/iteration_binding47_'+state+'_before_save.png')
 data=subprocess.run([magick,str(path),'txt:-'],capture_output=True,text=True,check=True).stdout
 points=set()
 for line in data.splitlines():
  m=re.match(r'(\d+),(\d+): \((\d+),(\d+),(\d+)',line)
  if not m:continue
  x,y,r,g,b=map(int,m.groups())
  green=r<60 and g>80 and b<60
  orange=r>220 and 100<g<230 and b<180
  if 64<=x<=457 and 69<=y<=318 and (green or orange):points.add((x,y))
 components=[]
 while points:
  pending=[points.pop()];component=[]
  while pending:
   x,y=pending.pop();component.append((x,y))
   for q in [(x-1,y),(x+1,y),(x,y-1),(x,y+1)]:
    if q in points:points.remove(q);pending.append(q)
  if len(component)>=4:components.append((min(x for x,y in component),max(x for x,y in component),min(y for x,y in component),max(y for x,y in component),len(component)))
 print(state,sorted(components))
