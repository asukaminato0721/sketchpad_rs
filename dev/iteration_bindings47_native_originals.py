"""Retain original binding groups and copies for Native open/save evidence."""
from pathlib import Path
import struct
entries=[('circle_rolling','个人专栏/方小庆作品/圆在并排圆上滚动(inRm).gsp',[9]),('circle_segment','个人专栏/方小庆作品/勾股螺旋(inRm).gsp',[13]),('polygon_scalar','个人专栏/侯仰顺作品/杨老师问题-迭代（蚂蚁）.gsp',[22]),('arc_spiral','个人专栏/向忠作品/黄金三角螺线.gsp',[20,25]),('ray_axis','个人专栏/贺基旭作品/数轴（hjx4882）.gsp',[16]),('line_reflection','未分类档/两直线间光线反射 .gsp',[27])]
for state,name,ids in entries:
 path=Path('../gsp-rs/tests/Samples')/name;d=path.read_bytes();Path('/tmp/iteration_binding47_original_'+state+'_input.gsp').write_bytes(d)
 o=4;ordinal=0;start=0
 while o<len(d):
  n,k=struct.unpack_from('<II',d,o);end=o+8+n+n%2
  if k==0x7d0:ordinal+=1;start=o
  if k==0x7d7 and ordinal in ids:
   Path('tests/fixtures/iteration_binding47_'+state+'_'+str(ordinal)+'_group.bin').write_bytes(d[start:end]);print(state,ordinal,d[start:end].hex())
  o=end
 htm=path.with_suffix('.htm');print(state,'original',path,'bytes',len(d),'pairedHTM',htm.exists())
