/* HIP-25 wallet shared state and utilities */
(function (W) {
  W.MAINNET_CHAIN_ID = "0";
  W.TESTNET = {
    address: "1Do17BuqMj5N4EZRuquXtoCCHFZpQoHyc2",
    password: "hip25test",
    fee: "0:247",
  };

  W.state = {
    diamonds: [],
    mortgageContracts: [],
    mortgageGlobal: null,
    height: 0,
    selected: new Set(),
    wasm: null,
    hip25Dev: true,
    signAddress: "",
  };

  W.$ = (id) => document.getElementById(id);

  W.txTimestamp = () => Math.floor(Date.now() / 1000);

  W.sdkOrigin = () => window.location.origin;

  W.log = (msg, cls = "") => {
    const el = W.$("log");
    const line = document.createElement("div");
    if (cls) line.className = cls;
    line.textContent = `[${new Date().toLocaleTimeString()}] ${msg}`;
    el.prepend(line);
  };

  W.parseAccountJson = (raw) => {
    if (!raw || raw.startsWith("[ERROR]")) throw new Error(raw || "account error");
    const trimmed = raw.trim();
    if (!trimmed.startsWith("{")) throw new Error("invalid account JSON");
    return JSON.parse(trimmed);
  };

  W.parseSdkJson = (raw) => {
    if (!raw || raw.startsWith("[ERROR]")) throw new Error(raw || "SDK error");
    const trimmed = raw.trim();
    if (!trimmed.startsWith("{")) throw new Error("SDK returned invalid JSON");
    return JSON.parse(trimmed);
  };

  W.hexToBytes = (hex) => {
    const out = new Uint8Array(hex.length / 2);
    for (let i = 0; i < out.length; i++) out[i] = parseInt(hex.substr(i * 2, 2), 16);
    return out;
  };

  W.splitDiamonds = (str) => {
    if (!str) return [];
    const w = 6;
    const s = str.replace(/\s/g, "");
    const out = [];
    for (let i = 0; i < s.length; i += w) out.push(s.slice(i, i + w));
    return out.filter((x) => x.length === w);
  };

  W.randomLendIdHex = () => {
    const bytes = new Uint8Array(14);
    bytes[0] = 0x4d;
    crypto.getRandomValues(bytes.subarray(1, 13));
    bytes[13] = 0x7a;
    return Array.from(bytes).map((b) => b.toString(16).padStart(2, "0")).join("");
  };
})(window.Hip25Wallet = window.Hip25Wallet || {});