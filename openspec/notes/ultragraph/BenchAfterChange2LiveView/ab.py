"""Compare one old run with one new run, row by row: the ratio and whether it exceeds 5 %."""
import re, sys
UNIT = {"ps": 1e-3, "ns": 1.0, "µs": 1e3, "ms": 1e6, "s": 1e9}
def parse_log(path):
    rows, pending = {}, None
    for line in open(path, encoding="utf-8"):
        m = re.match(r"^(\S+)\s*(?:time:\s*\[(.*)\])?\s*$", line)
        if m and "time:" in line and "/" in m.group(1):
            rows[m.group(1)] = est(m.group(2)); pending = None; continue
        if m and m.group(2) is None and "/" in m.group(1) and not line.startswith(" ") and "Benchmarking" not in line:
            pending = m.group(1); continue
        t = re.match(r"^\s+time:\s*\[(.*)\]", line)
        if t and pending:
            rows[pending] = est(t.group(1)); pending = None
    return rows
def est(inner):
    p = inner.split(); return float(p[2]) * UNIT[p[3]]
def fmt(ns):
    if ns < 1e3: return f"{ns:.2f} ns"
    if ns < 1e6: return f"{ns/1e3:.2f} µs"
    return f"{ns/1e6:.3f} ms"
old = parse_log(sys.argv[1]); new = parse_log(sys.argv[2])
print("| Row | Before the Live state | Now | Δ |")
print("|---|---:|---:|---:|")
beyond = []
for k in old:
    if k not in new: continue
    d = (new[k] / old[k] - 1) * 100
    flag = " **" if abs(d) > 5 else ""
    print(f"| `{k}` | {fmt(old[k])} | {fmt(new[k])} | {d:+.1f} %{flag} |")
    if abs(d) > 5: beyond.append((k, d))
print(); print(len(beyond), "rows beyond 5 %:", [(k, round(d, 1)) for k, d in beyond])
