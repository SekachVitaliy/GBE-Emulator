#!/usr/bin/env python3
"""worst.py <results_dir> [--line NAME]: per-subsystem worst testcases, or a one-line scores.md entry."""
import json, sys, os, collections
d = sys.argv[1]
s = json.load(open(os.path.join(d, "summary.json")))
sec = s["sections"]
if "--line" in sys.argv:
    n = sys.argv[sys.argv.index("--line") + 1]
    f = lambda k: sec[k]["score"]
    print(f"- {s['graded_at']} {n} sha={s['candidate_sha256'][:8]} procedural={f('procedural'):.4f} "
          f"replay={f('replay'):.4f} audio={f('audio'):.4f} overall={s['overall']:.4f} dir={d}")
    sys.exit()
print(f"overall {s['overall']:.4f}  " + "  ".join(f"{k}={v['score']:.4f}(w{v['weight']})" for k, v in sec.items()))
g = collections.defaultdict(list)
for t, v in s["per_testcase"].items():
    g[(v["section"], v["subsystem"])].append((v["video_score"], v.get("audio_score"), t))
for (sc, sub), L in sorted(g.items()):
    w = None
    print(f"\n[{sc}/{sub}] subsystem score {sec[sc]['subsystems'].get(sub, 0):.4f}")
    for vs, a, t in sorted(L, key=lambda x: x[0])[:5]:
        try:
            j = json.load(open(os.path.join(d, t + ".json")))
            ex = f" first_diverge={j.get('first_diverge_frame')} diverging={j.get('diverging_frames')}/{j.get('n_frames')}"
        except Exception:
            ex = ""
        print(f"  {t:28s} video={vs:.4f}" + (f" audio={a:.4f}" if a is not None else "") + ex)
