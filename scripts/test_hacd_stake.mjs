import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import vm from "vm";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const pkgDir = path.join(root, "target", "debug", "pkg");
const js = fs.readFileSync(path.join(pkgDir, "hacash_sdk.js"), "utf8");
const wasm = fs.readFileSync(path.join(pkgDir, "hacash_sdk_bg.wasm"));

const ctx = vm.createContext({
  console,
  WebAssembly,
  TextDecoder,
  TextEncoder,
  performance: globalThis.performance,
  Date,
  Math,
  Reflect,
  ArrayBuffer,
  Uint8Array,
  Int32Array,
  Float64Array,
  Promise,
  URL,
  location: { href: "http://localhost/" },
  document: { currentScript: { src: "http://localhost/hacash_sdk.js" } },
});
vm.runInContext(js.replace("let wasm_bindgen", "var wasm_bindgen"), ctx);
await ctx.wasm_bindgen({ module_or_path: wasm });

const raw = ctx.wasm_bindgen.hacd_stake(1n, "hip25test", "WTYUIA", "0:247", 1718496000n);
if (raw.startsWith("[ERROR]")) throw new Error(raw);
const tx = JSON.parse(raw.trim().startsWith("{") ? raw.trim() : `{${raw.trim()}}`);
console.log("hacd_stake OK", tx.tx_hash);

const base = process.env.HACASH_RPC || "http://127.0.0.1:8083";
try {
  const body = Buffer.from(tx.tx_body, "hex");
  const res = await fetch(`${base}/submit/transaction`, {
    method: "POST",
    headers: { "Content-Type": "application/octet-stream" },
    body,
  });
  const out = await res.json();
  if (out.ret !== 0) throw new Error(JSON.stringify(out));
  console.log("submit OK", out.hash);
} catch (e) {
  console.log("(submit skipped — node not running)", e.message);
}