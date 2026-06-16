/* Local WASM signing address resolution */
(function (W) {
  W.resolveSigningAddress = async () => {
    const pass = W.$("prikey").value.trim();
    const el = W.$("signAddress");
    const warn = W.$("addressWarn");
    if (!pass) {
      W.state.signAddress = "";
      el.value = "";
      warn.textContent = "";
      return "";
    }
    if (!W.state.wasm?.create_account_by) {
      el.value = "(rebuild WASM: scripts/build_wallet_sdk.ps1)";
      return "";
    }
    try {
      const acc = W.parseAccountJson(W.state.wasm.create_account_by(pass));
      W.state.signAddress = acc.address || "";
      el.value = W.state.signAddress;
      W.updateAddressMismatchWarning();
      return W.state.signAddress;
    } catch (e) {
      W.state.signAddress = "";
      el.value = "";
      warn.textContent = e.message;
      warn.className = "meta err";
      return "";
    }
  };

  W.updateAddressMismatchWarning = () => {
    const warn = W.$("addressWarn");
    const portfolio = W.$("address").value.trim();
    const signer = W.state.signAddress;
    if (!portfolio || !signer) {
      warn.textContent = "";
      return;
    }
    if (portfolio !== signer) {
      warn.textContent = `Mismatch: portfolio is ${portfolio} but password signs as ${signer}. Transactions will fail until they match.`;
      warn.className = "meta err";
      return;
    }
    warn.textContent = "Portfolio address matches signing address.";
    warn.className = "meta ok";
  };

  W.loadSdkScript = () =>
    new Promise((resolve, reject) => {
      if (typeof wasm_bindgen === "function") return resolve();
      const s = document.createElement("script");
      s.src = `${W.sdkOrigin()}/pkg/hacash_sdk.js`;
      s.onload = () => resolve();
      s.onerror = () => reject(new Error("failed to load /pkg/hacash_sdk.js"));
      document.head.appendChild(s);
    });

  W.initWasmSdk = async () => {
    const el = W.$("sdkStatus");
    try {
      await W.loadSdkScript();
      const wasmUrl = `${W.sdkOrigin()}/pkg/hacash_sdk_bg.wasm`;
      await wasm_bindgen(wasmUrl);
      W.state.wasm = wasm_bindgen;
      el.textContent = "WASM SDK: ready — stake / unstake / mortgage (password stays in browser)";
      el.className = "meta ok";
      W.updateActionButtons();
      await W.resolveSigningAddress();
    } catch (e) {
      el.textContent = `WASM SDK: ${e.message}`;
      el.className = "meta err";
    }
  };
})(window.Hip25Wallet);