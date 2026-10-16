"""Aggregate three full runs; compare every row that has an incumbent against it by the D6 match
rule (within when the mean is at most the incumbent's times one plus the larger of 5 % and the
incumbent row's spread; faster when below by more than the spread; slower otherwise)."""
import re, sys, statistics
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
def parse_table(path, section_start, section_end=None):
    """Rows `| name | n | ... | mean | spread |` from a markdown section: name/n -> (mean ns, spread %)."""
    text = open(path, encoding="utf-8").read()
    a = text.index(section_start); b = text.index(section_end, a) if section_end else len(text)
    out = {}
    for line in text[a:b].splitlines():
        m = re.match(r"\| `(\S+)` \| ([^|]+?) \|(.*)", line)
        if not m: continue
        cells = [c.strip() for c in m.group(3).split("|")]
        # find the mean cell: the first cell after the three runs that has a unit, then the spread cell
        vals = [c for c in cells if re.match(r"[\d.]+ (ns|µs|ms)( \(median\))?$", c)]
        spreads = [c for c in cells if re.match(r"[\d.]+ %$", c)]
        if len(vals) >= 4 and spreads:
            mean = vals[3]; v = float(mean.split()[0]) * UNIT[mean.split()[1]]
            n = m.group(2).replace(",", "")
            out[(m.group(1), n)] = (v, float(spreads[0].split()[0]))
    return out
runs = [parse_log(p) for p in sys.argv[1:4]]
ids = [k for k in runs[0] if all(k in r for r in runs)]
agg = {}
for k in ids:
    v = [r[k] for r in runs]; mean = sum(v) / 3; spread = (max(v) - min(v)) / mean * 100
    agg[k] = (v, mean, spread, statistics.median(v) if spread > 15 else mean)
# incumbents: change-1 after (crud/linear/biconnectivity rows), group-2 incumbent (storage rows), group 5 (live rows)
inc = {}
inc.update(parse_table(sys.argv[4], "## After the update", "## Every move"))
inc.update({k: v for k, v in parse_table(sys.argv[5], "### Group 2: the incumbent", "### Group 3").items()})
inc.update({k: v for k, v in parse_table(sys.argv[5], "### Group 5: the Live decision", "**Decision (task 5.2)").items()})
print("| Row | Run 1 | Run 2 | Run 3 | Mean | Spread | Incumbent | Its spread | Δ | Verdict |")
print("|---|---:|---:|---:|---:|---:|---:|---:|---:|---|")
counts = {"within spread": 0, "slower": 0, "faster": 0, "new": 0}
for k in ids:
    v, mean, spread, centre = agg[k]
    name, _, n = k.partition("/")
    key = (name, n.replace(",", ""))
    label = fmt(centre) + (" (median)" if spread > 15 else "")
    base = inc.get(key) or inc.get((k.replace("/", "/", 1).split("/")[0], n))
    if base is None:
        # live rows are keyed by the full id in group 5's table
        base = inc.get((k, ""))
    if base is None:
        counts["new"] += 1
        print(f"| `{k}` | {fmt(v[0])} | {fmt(v[1])} | {fmt(v[2])} | {label} | {spread:.1f} % | new | | | new |")
        continue
    bmean, bspread = base
    tol = max(5.0, bspread)
    delta = (centre / bmean - 1) * 100
    if centre <= bmean * (1 + tol / 100) and centre >= bmean * (1 - tol / 100): verdict = "within spread"
    elif centre < bmean: verdict = "faster"
    else: verdict = "slower"
    counts[verdict] += 1
    print(f"| `{k}` | {fmt(v[0])} | {fmt(v[1])} | {fmt(v[2])} | {label} | {spread:.1f} % | {fmt(bmean)} | {bspread:.1f} % | {delta:+.1f} % | {verdict} |")
print()
print(counts)
