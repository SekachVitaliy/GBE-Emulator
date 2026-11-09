#!/usr/bin/env bash
# usage: cmpframe.sh romname(homebrew) replayname N  -> /tmp ppm pair + diff count
D=$(cd "$(dirname "$0")/.." && pwd); R=$D/../gba-eval/corpus; T=/tmp/gba-emu-cmp; mkdir -p $T
node $D/scripts/refframe.mjs $R/roms/homebrew/$1.gba $R/replays/$2.txt $3 $T/ref.ppm
$D/target/release/examples/run $R/roms/homebrew/$1.gba $3 $T/our.ppm $R/replays/$2.txt >/dev/null
python3 - <<PY
a=open("$T/ref.ppm","rb").read()[15:];b=open("$T/our.ppm","rb").read()[15:]
d=[i//3 for i in range(0,len(a),3) if a[i:i+3]!=b[i:i+3]]
print("diff px",len(d), "rows", (min(x//240 for x in d),max(x//240 for x in d)) if d else None)
PY
