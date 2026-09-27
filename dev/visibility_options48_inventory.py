"""Read-only raw visibility option / paired HTM audit; no semantic guessing."""
from pathlib import Path
import struct,collections,json,re
root=Path('../gsp-rs')
summary=collections.Counter(); matched=collections.Counter();examples={}; selected={}; count=0
for path in sorted(root.rglob('*.gsp')):
 data=path.read_bytes(); off=4; ordinal=0; group=[]
 htm=path.with_suffix('.htm');text=htm.read_text(errors='replace') if htm.exists() else ''
 lines=[l.strip() for l in text.splitlines() if any(k+'(' in l for k in ['HideButton','ShowButton','ToggleVisibilityButton'])]
 while off<len(data):
  if off+8>len(data):break
  length,kind=struct.unpack_from('<II',data,off);end=off+8+length; payload=data[off+8:end]; start=off;off=end+length%2
  if off>len(data):break
  if kind==0x7d0:group=[];ordinal+=1
  group.append((kind,payload,start,off))
  if kind!=0x7d7 or not group or group[0][0]!=0x7d0 or len(group[0][1])<2:continue
  if struct.unpack_from('<H',group[0][1])[0]!=62:continue
  records={k:p for k,p,_,_ in group};action=records.get(0x906,b'')
  if len(action)!=24:continue
  words=struct.unpack_from('<4H',action,12)
  if words[0] not in (0,1):continue
  label=records.get(0x7d5,b'');name=label[24:24+struct.unpack_from('<H',label,22)[0]].decode('utf8','replace') if len(label)>=24 else ''
  refs=records.get(0x7d2,b'');ids=list(struct.unpack('<'+'I'*(len(refs)//4),refs))[1:] if len(refs)>=4 and len(refs)%4==0 else []
  found=[line for line in lines if any(name in value.split("|") for value in re.findall(r"'([^']*)'",line))]
  funcs=sorted({f for f in ['HideButton','ShowButton','ToggleVisibilityButton'] if any(f+'(' in line for line in found)})
  summary[words]+=1;count+=1
  if len(funcs)==1:matched[(words,funcs[0])]+=1
  info=dict(file=str(path),ordinal=ordinal,words=words,label=name,refs=ids,anchor=struct.unpack_from('<2h',action,20),header=group[0][1].hex(),action=action.hex(),records=[(hex(k),len(p)) for k,p,_,_ in group],htm=str(htm) if text else None,matching_htm=found)
  examples.setdefault(words,[])
  if len(examples[words])<3 or len(funcs)==1 and not any(e['matching_htm'] for e in examples[words]):examples[words].append(info)
  score=(bool(name),len(funcs)==1,bool(text))
  if words not in selected or score>selected[words][0]:
   selected[words]=(score,info,data[group[0][2]:group[-1][3]])
for words,(score,info,raw) in selected.items():
 out=Path('tests/fixtures')/f'visibility48_original_a{words[0]}_o{words[1]}_group.bin'
 out.write_bytes(raw)
 info['group_fixture']=str(out)
 print('SELECTED',json.dumps(info,ensure_ascii=False))
Path('/tmp/visibility48_selected.json').write_text(json.dumps([info for _,info,_ in selected.values()],ensure_ascii=False,indent=2))
print('RAW TOTAL',count)
for words,n in sorted(summary.items()):
 print('RAW',words,n,'HTM',[(key[1],c) for key,c in matched.items() if key[0]==words])
 for info in examples[words]: print(json.dumps(info,ensure_ascii=False))
