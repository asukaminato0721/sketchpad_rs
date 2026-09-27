"""Retain only completed Native probes, using terminal DONE lines in probe logs."""
from pathlib import Path
logs=['click_queue','overlap_modes','priority_queue','paths_queue','label_queue','api_queue']
for logname in logs:
 log=Path('/tmp/movement46_'+logname+'.log')
 if not log.exists():continue
 for line in log.read_text().splitlines():
  if not line.startswith('DONE '):continue
  state=line[5:];prefix='movement46_'+state
  for suffix in ['input','clicked']:
   source=Path('/tmp/'+prefix+'_'+suffix+'.gsp')
   name=prefix+('_input' if suffix=='input' else '_clicked_native_verified')+'.gsp'
   if state=='graph' and suffix=='clicked':name=prefix+'_rejected_native_verified.gsp'
   Path('tests/fixtures/'+name).write_bytes(source.read_bytes())
  capture=Path('/tmp/'+prefix+('_label_before_save.png' if 'label' in state else '_click_correct_after_action_click.png'))
  if capture.exists():Path('tests/fixtures/'+prefix+'_native_verified.png').write_bytes(capture.read_bytes())
  print(state)
