// usage: node divs.mjs rom replay [maxframes] -> frames where cand != ref
import fs from 'fs';
async function load(p) { const m = await WebAssembly.compile(fs.readFileSync(p)); const env = {}; for (const i of WebAssembly.Module.imports(m)) { env[i.module] ??= {}; env[i.module][i.name] = () => 0; } return (await WebAssembly.instantiate(m, env)).exports; }
const A = new URL('../../', import.meta.url).pathname;
const r = await load(A + 'gba-eval/reference/mesen.wasm'), c = await load((process.env.CAND || A + 'gba-emu/target/wasm32-unknown-unknown/release/gba_emu.wasm'));
const romb = fs.readFileSync(process.argv[2]);
const ev = fs.readFileSync(process.argv[3], 'utf8').split('\n').filter(l => l.trim() && !l.startsWith('#')).map(l => l.split(/\s+/)).map(w => [+w[0], parseInt(w[1], 16)]);
const N = +(process.argv[4] || 600);
for (const x of [r, c]) { x.emu_init(); new Uint8Array(x.memory.buffer, x.emu_rom_buffer(), romb.length).set(romb); x.emu_load_rom(romb.length); }
r.emu_set_keys(0); for (let i = 0; i < 6; i++) r.emu_run_frame();
let k = 0, cur = 0, shown = 0;
for (let f = 0; f < N; f++) {
  while (k < ev.length && ev[k][0] <= f) cur = ev[k++][1];
  r.emu_set_keys(cur); c.emu_set_keys(cur); r.emu_run_frame(); c.emu_run_frame();
  const a = new Uint32Array(r.memory.buffer, r.emu_framebuffer(), 38400), b = new Uint32Array(c.memory.buffer, c.emu_framebuffer(), 38400);
  let d = 0, y0 = 999, y1 = -1; for (let i = 0; i < 38400; i++) if (a[i] !== b[i]) { d++; const y = (i / 240) | 0; if (y < y0) y0 = y; y1 = y; }
  if (d && shown++ < 12) console.log('frame', f, 'diff', d, 'rows', y0, y1);
}
console.log('shown', shown);
