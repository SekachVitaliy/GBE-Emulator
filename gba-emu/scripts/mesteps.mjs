import fs from 'fs';
async function load(path) { const m = await WebAssembly.compile(fs.readFileSync(path)); const env = {}; for (const i of WebAssembly.Module.imports(m)) { env[i.module] ??= {}; env[i.module][i.name] = () => 0; } return (await WebAssembly.instantiate(m, env)).exports; }
const x = await load(new URL('../../gba-eval/reference/mesen.wasm', import.meta.url).pathname);
const romb = fs.readFileSync(process.argv[2]); const n = +process.argv[3];
x.emu_init(); new Uint8Array(x.memory.buffer, x.emu_rom_buffer(), romb.length).set(romb); x.emu_load_rom(romb.length);
let out = []; let prev = BigInt(x.mesen_master_clock());
for (let i = 0; i < n; i++) { const r = x.mesen_exec_one(); out.push((x.mesen_pc()>>>0).toString(16) + ' ' + r); }
process.stdout.write(out.join('\n') + '\n');
