"""Retain only terminal-DONE visibility controls and inspect their actual saves."""
from pathlib import Path
import re,shutil,json,importlib.util
spec=importlib.util.spec_from_file_location('inspect48','dev/visibility_options48_inspect.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
logs=['initial','remaining','bit_pair','caption_bits','caption_full','caption_prefix','missing_child','categories']
states=[]
for name in logs:
 p=Path('/tmp')/f'visibility48_{name}_native.log'
 if p.exists():states+=re.findall(r'^DONE (\w+)$',p.read_text(),re.M)
for state in dict.fromkeys(states):
 source=Path('/tmp')/f'visibility48_{state}_input.gsp';opened=Path('/tmp')/f'visibility48_{state}_opened.gsp'
 assert source.read_bytes()==opened.read_bytes(),state
 shutil.copyfile(source,Path('tests/fixtures')/f'visibility48_{state}_input.gsp')
 result=dict(state=state,bytes=source.stat().st_size,initial=m.inspect(source),steps=[])
 for step in (1,2):
  path=Path('/tmp')/f'visibility48_{state}_step{step}.gsp';png=path.with_suffix('.png')
  assert path.exists() and png.exists(),state
  shutil.copyfile(path,Path('tests/fixtures')/f'visibility48_{state}_step{step}_native_verified.gsp')
  shutil.copyfile(png,Path('tests/fixtures')/f'visibility48_{state}_step{step}_native_verified.png')
  result['steps'].append(m.inspect(path))
 print(json.dumps(result,ensure_ascii=False))
print('CONTROLS',len(set(states)))
