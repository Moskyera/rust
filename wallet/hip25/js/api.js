/* RPC client */
(function (W) {
  W.apiBase = () => {
    let b = W.$("rpc").value.trim().replace(/\/$/, "");
    if (!b) b = W.sdkOrigin();
    return b;
  };

  W.explainSubmitError = (msg) => {
    if (msg.includes("loan amount must be")) {
      return `${msg} — re-select HACD or use Calculate loan to sync loan amount with bid-burn sum`;
    }
    if (msg.includes("do hac_sub error") && msg.includes("not enough")) {
      const portfolio = W.$("address").value.trim() || "portfolio address";
      const signer = W.state.signAddress || "signing address";
      return `${msg} — ensure password matches ${portfolio} (signs as ${signer}) and HAC balance covers origination burn + tx fee`;
    }
    if (msg.includes("mortgage owner contract index full")) {
      const max = W.state.mortgageGlobal?.owner_index_max ?? 64;
      return `${msg} — maximum ${max} active mortgage contracts per address`;
    }
    return msg;
  };

  W.apiGet = async (path, params = {}) => {
    const qs = new URLSearchParams(params).toString();
    const url = `${W.apiBase()}/${path}${qs ? "?" + qs : ""}`;
    let r;
    try {
      r = await fetch(url);
    } catch (e) {
      throw new Error(`RPC unreachable at ${W.apiBase()} — run START_WALLET.bat and keep HIP25-FULLNODE open`);
    }
    const text = await r.text();
    let j;
    try {
      j = JSON.parse(text);
    } catch {
      throw new Error(text.trim() || `RPC error ${r.status}`);
    }
    if (j.ret !== 0) throw new Error(W.explainSubmitError(j.err || JSON.stringify(j)));
    return j;
  };

  W.apiPost = async (path, body, params = {}) => {
    const qs = new URLSearchParams(params).toString();
    const url = `${W.apiBase()}/${path}${qs ? "?" + qs : ""}`;
    const bytes = body instanceof Uint8Array ? body : new TextEncoder().encode(body);
    const r = await fetch(url, { method: "POST", body: bytes });
    const text = await r.text();
    let j;
    try {
      j = JSON.parse(text);
    } catch {
      throw new Error(text.trim() || `RPC error ${r.status}`);
    }
    if (j.ret !== 0) throw new Error(W.explainSubmitError(j.err || JSON.stringify(j)));
    return j;
  };

  W.warnRpcOriginMismatch = () => {
    const api = W.apiBase();
    const sdk = W.sdkOrigin();
    if (api !== sdk) {
      W.log(`RPC URL (${api}) differs from wallet origin (${sdk}); WASM SDK always loads from wallet origin`, "err");
    }
  };
})(window.Hip25Wallet);