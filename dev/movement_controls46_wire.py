"""Append Native Movement template with explicitly selected reference pairs."""
from pathlib import Path
import struct, subprocess, sys

def records(d):
 # Full GSP files have the established four-byte preamble.
 o=4
 while o<len(d):
  n,k=struct.unpack_from('<II',d,o);e=o+8+n+n%2
  yield o,k,d[o+8:o+8+n],e
  o=e

def rec(k,p):return struct.pack('<II',len(p),k)+p+b'\0'*(len(p)%2)
for state,pairs in [('overlap',[(1,2),(2,3)]),('overlap_slow',[(1,2),(2,3)]),('segment',[(4,5)]),('circle',[(4,5)]),('ray',[(4,5)]),('line',[(4,5)]),('graph',[(2,7),(4,8)]),('parameter',[(1,2),(2,3)]),('parameter_mismatch',[(1,2),(2,3)]),('arc',[(5,6)]),('polygon',[(5,6)])]:
 if len(sys.argv)>1 and state not in sys.argv[1:]:continue
 subprocess.run(['/tmp/movement_controls46_generate',state],check=True)
 d=bytearray(Path('/tmp/movement46_'+state+'_base.gsp').read_bytes())
 rs=list(records(d));pre=next(r for r in rs if r[1]==0x3e8);count=sum(r[1]==0x7d0 for r in rs);struct.pack_into('<I',d,pre[0]+8+24,count+1)
 last=max(r[3] for r in rs if r[1]==0x7d7);insert=next((r[0] for r in rs if r[0]>=last and r[1] in (0x2328,0x232d,0x3e9)),len(d))
 raw=Path('assets/action_movement_group.bin').read_bytes();out=b''
 o=0
 while o<len(raw):
  n,k=struct.unpack_from('<II',raw,o);p=bytearray(raw[o+8:o+8+n]);o+=8+n+n%2
  if k==0x7d2:p=bytearray(struct.pack('<'+'I'*(1+2*len(pairs)),2*len(pairs),*(v for pair in pairs for v in pair)))
  if k==0x906:struct.pack_into('<H',p,14,0 if state.endswith('slow') else 3)
  out+=rec(k,p)
 d[insert:insert]=out
 Path('/tmp/movement46_'+state+'_input.gsp').write_bytes(d)
