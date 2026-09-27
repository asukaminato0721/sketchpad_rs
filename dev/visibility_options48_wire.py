"""Isolate original option payloads on two mixed-visibility Free targets.

Retain option words and all original opaque payloads. Explicitly adjust only the
reference list, hidden header byte, screen anchor, and (when absent) caption.
"""
from pathlib import Path
import struct,json

def records(data,start=4):
 offset=start
 while offset<len(data):
  length,kind=struct.unpack_from('<II',data,offset);end=offset+8+length
  yield offset,kind,data[offset+8:end],data[end:end+length%2],end+length%2
  offset=end+length%2

def record(kind,payload,padding=b''):
 return struct.pack('<II',len(payload),kind)+payload+(padding or b'\0')[:len(payload)%2]

for info in json.loads(Path('/tmp/visibility48_selected.json').read_text()):
 action,option,word16,word18=info['words'];state=f'a{action}_o{option}'
 if word16 or word18:continue
 raw=Path(info['group_fixture']).read_bytes();group=list(records(raw,0));out=b''
 has_label=any(kind==0x7d5 for _,kind,*_ in group)
 for _,kind,payload,padding,_ in group:
  payload=bytearray(payload)
  if kind==0x7d0:payload[2]=0
  if kind==0x7d2:payload=bytearray(struct.pack('<3I',2,1,2))
  if kind==0x906:struct.pack_into('<2h',payload,20,10,5)
  if kind==0x7d8 and not has_label:
   template=list(records(Path('assets/action_hide_show_two_points_group.bin').read_bytes(),0))
   label=bytearray(next(p for _,k,p,_,_ in template if k==0x7d5));text=b'State'
   label=label[:22]+struct.pack('<H',len(text))+text+b'\0'
   out+=record(0x7d5,label)
  out+=record(kind,payload,padding)
 base=bytearray(Path('/tmp/visibility48_base.gsp').read_bytes());rs=list(records(base))
 pre=next(r for r in rs if r[1]==0x3e8);count=sum(r[1]==0x7d0 for r in rs)
 struct.pack_into('<I',base,pre[0]+8+24,count+1)
 last=max(r[4]for r in rs if r[1]==0x7d7)
 insert=next((r[0]for r in rs if r[0]>=last and r[1]in(0x2328,0x232d,0x3e9)),len(base))
 base[insert:insert]=out
 Path(f'/tmp/visibility48_{state}_input.gsp').write_bytes(base)
 setup=Path(f'/tmp/visibility48_{state}_setup.sh')
 setup.write_text(f'''#!/usr/bin/env bash
set -euo pipefail
xdotool key --clearmodifiers ctrl+s
sleep 0.5
cp /tmp/visibility48_{state}_native_work.gsp /tmp/visibility48_{state}_opened.gsp
for step in 1 2; do
 xdotool mousemove 85 84 click 1
 sleep 0.7
 import -window root /tmp/visibility48_{state}_step${{step}}.png
 xdotool key --clearmodifiers ctrl+s
 sleep 0.5
 cp /tmp/visibility48_{state}_native_work.gsp /tmp/visibility48_{state}_step${{step}}.gsp
done
''')
 print(state,'label_added',not has_label,info['file'],info['ordinal'])
