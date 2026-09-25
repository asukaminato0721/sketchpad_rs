#!/usr/bin/env python3
"""Replace the original one-row table with an original two-row table."""

from pathlib import Path


def records(data: bytes):
    at = 4
    while at < len(data):
        size = int.from_bytes(data[at : at + 4], "little")
        kind = int.from_bytes(data[at + 4 : at + 8], "little")
        end = at + 8 + size + size % 2
        if end > len(data):
            raise ValueError("truncated record")
        yield kind, at, end
        at = end


def table_group(data: bytes):
    entries = list(records(data))
    start = [r for r in entries if r[0] == 0x7D0][1][1]
    end = [r for r in entries if r[0] == 0x7D7][1][2]
    return start, end


fixtures = Path("tests/fixtures")
frame = (fixtures / "plot_points_table_frame.gsp").read_bytes()
two = (fixtures / "table_parameter_two_rows.gsp").read_bytes()
start, end = table_group(frame)
two_start, two_end = table_group(two)
output = frame[:start] + two[two_start:two_end] + frame[end:]
list(records(output))
(fixtures / "plot_points_table_two_rows_base.gsp").write_bytes(output)
