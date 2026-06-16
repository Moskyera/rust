import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import vm from "vm";

const root = path.join(path.dirname(fileURLToPath(import.meta.url)), "..");
const pkgDir = path.join(root, "target", "debug", "pkg");
const js = fs.readFileSync(path.join(pkgDir, "hacash_sdk.js"), "utf8");
const wasm = fs.readFileSync(path.join(pkgDir, "hacash_sdk_bg.wasm"));

const kind = parseInt(process.env.HIP2_KIND || "15", 10);
const pass = process.env.HIP2_PASS || "hip25test";
const lendId = process.env.HIP2_LEND_ID || "4801000000000000000000000032";
const diamond = process.env.HIP2_DIAMOND || "WTYUIA";
const loan = process.env.HIP2_LOAN || "100";
const borrowPeriod = parseInt(process.env.HIP2_BORROW_PERIOD || "5", 10);
const fee = "0:247";
const ts = 1718496000n;
const base = process.env.HACASH_RPC || "http://127.0.0.1:8083";

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

let raw;
if (kind === 15) {
  raw = ctx.wasm_bindgen.hacd_mortgage_open(
    1n, pass, lendId, diamond, loan, borrowPeriod, fee, ts
  );
} else if (kind === 16) {
  const qRes = await fetch(
    `${base}/query/mortgage/contract?id=${lendId}&redeemer=1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2`
  );
  const q = await qRes.json();
  if (q.ret !== 0) throw new Error(JSON.stringify(q));
  const ransom = q.min_ransom || q.data?.min_ransom;
  if (!ransom) throw new Error("no min_ransom from quote");
  raw = ctx.wasm_bindgen.hacd_mortgage_redeem(1n, pass, lendId, ransom, fee, ts);
} else {
  throw new Error(`unsupported kind ${kind}`);
}

if (raw.startsWith("[ERROR]")) throw new Error(raw);
const tx = JSON.parse(raw.trim().startsWith("{") ? raw.trim() : `{${raw.trim()}}`);
console.error(`WASM kind ${kind} OK tx_hash=${tx.tx_hash}`);

const body = Buffer.from(tx.tx_body, "hex");
const res = await fetch(`${base}/submit/transaction`, {
  method: "POST",
  headers: { "Content-Type": "application/octet-stream" },
  body,
});
const out = await res.json();
if (out.ret !== 0) throw new Error(JSON.stringify(out));
console.log(out.hash);