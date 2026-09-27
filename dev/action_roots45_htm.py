"""Read-only original action groups and complete paired HTM evidence."""
from pathlib import Path
import struct

paths = [
    Path('../gsp-rs/tests/Samples/个人专栏/况永胜作品/正方体的展开（3D效果）.gsp'),
    Path('../gsp-rs/tests/Samples/个人专栏/李忠平作品/金华2010-24题(百年孤独)10.8.9.gsp'),
    Path('../gsp-rs/tests/Samples/个人专栏/周维波作品/2010福建宁德(飞狐制作).gsp'),
    Path('../gsp-rs/tests/Samples/简易数轴与坐标系/数轴.gsp'),
]
for path in paths:
    print('\nGSP:', path)
    data = path.read_bytes()
    offset, ordinal, group = 4, 0, []
    while offset < len(data):
        length, kind = struct.unpack_from('<II', data, offset)
        payload = data[offset + 8:offset + 8 + length]
        offset += 8 + length + length % 2
        if kind == 0x7d0:
            group = []
            ordinal += 1
        group.append((kind, payload))
        if kind != 0x7d7 or not group or group[0][0] != 0x7d0:
            continue
        if struct.unpack_from('<H', group[0][1])[0] != 62:
            continue
        records = dict(group)
        action = records.get(0x906, b'')
        refs = records.get(0x7d2, b'')
        label = records.get(0x7d5, b'')
        name = label[24:24 + struct.unpack_from('<H', label, 22)[0]].decode('utf8', 'replace') if len(label) >= 24 else '<missing07d5>'
        print('object', ordinal, 'header', group[0][1].hex(),
              'refs-with-count', list(struct.unpack('<' + 'I' * (len(refs) // 4), refs)),
              'action', action.hex(), 'label', repr(name),
              'records', [(hex(k), len(p)) for k, p in group])
    htm = path.with_suffix('.htm')
    print('HTM:', htm, 'exists', htm.exists())
    print(htm.read_text(errors='replace') if htm.exists() else '<missing>')
