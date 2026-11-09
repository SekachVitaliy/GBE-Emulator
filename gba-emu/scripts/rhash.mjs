import fs from 'fs';
const m = await WebAssembly.compile(fs.readFileSync(new URL('../../gba-eval/reference/mesen.wasm', import.meta.url).pathname)); const env = {}; for (const i of WebAssembly.Module.imports(m)) { env[i.module] ??= {}; env[i.module][i.name] = () => 0; }
const x = (await WebAssembly.instantiate(m, env)).exports;
const romb = fs.readFileSync(process.argv[2]);
const ev = fs.readFileSync(process.argv[3], 'utf8').split('\n').filter(l => l.trim() && !l.startsWith('#')).map(l => l.split(/\s+/)).map(w => [+w[0], parseInt(w[1], 16)]);
const N = +process.argv[4];
x.emu_init(); new Uint8Array(x.memory.buffer, x.emu_rom_buffer(), romb.length).set(romb); x.emu_load_rom(romb.length);
x.emu_set_keys(0); for (let i = 0; i < 6; i++) x.emu_run_frame();
let k = 0, cur = 0; const out = [];
for (let f = 0; f < N; f++) { while (k < ev.length && ev[k][0] <= f) cur = ev[k++][1]; x.emu_set_keys(cur); x.emu_run_frame();
  const fb = new Uint32Array(x.memory.buffer, x.emu_framebuffer(), 38400); let h = 1469598103934665603n; const M = (1n << 64n) - 1n; for (let i = 0; i < 38400; i++) h = ((h ^ BigInt(fb[i])) * 1099511628211n) & M; out.push(h.toString(16).padStart(16, '0')); }
console.log(out.join('\n'));
