"""Inspect original Native saves without assigning meanings to option bits."""
from pathlib import Path
import struct,json,sys

def inspect(path):
 data=Path(path).read_bytes();off=4;ordinal=0;group=[];result=[]
 while off<len(data):
  length,kind=struct.unpack_from('<II',data,off);payload=data[off+8:off+8+length];off+=8+length+length%2
  if kind==0x7d0:ordinal+=1;group=[]
  group.append((kind,payload))
  if kind!=0x7d7 or not group or group[0][0]!=0x7d0:continue
  r=dict(group);header=group[0][1];code=struct.unpack_from('<H',header)[0]
  label=r.get(0x7d5,b'');name=label[24:24+struct.unpack_from('<H',label,22)[0]].decode('utf8','replace')if len(label)>=24 else None
  entry=dict(id=ordinal,code=code,header_visibility=header[2],label=name,records=[(hex(k),len(p))for k,p in group])
  if 0x906 in r:entry['action_words']=struct.unpack_from('<4H',r[0x906],12)
  result.append(entry)
 return result
for path in sys.argv[1:]:print(json.dumps(dict(file=path,objects=inspect(path)),ensure_ascii=False))
