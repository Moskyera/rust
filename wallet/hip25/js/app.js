/* Event bindings and bootstrap */
(function (W) {
  W.bootstrap = async () => {
    await W.initWasmSdk();
    try {
      const latest = await W.apiGet("query/latest");
      if (latest.chain_id !== undefined) W.$("chainId").value = String(latest.chain_id);
      W.state.hip25Dev = !!latest.hip25_dev;
      if (latest.hip25_dev) {
        W.$("networkTag").textContent = "HIP-25 testnet";
        if (!W.$("address").value.trim()) {
          W.$("address").value = W.TESTNET.address;
          W.$("fee").value = W.TESTNET.fee;
          W.$("chainId").value = "1";
        }
        if (!W.$("lendId").value.trim()) W.$("lendId").value = W.randomLendIdHex();
        setTimeout(() => W.loadPortfolio().catch((e) => W.log(e.message, "err")), 800);
      } else {
        W.$("btnTestnet").style.display = "none";
        W.$("chainId").value = W.MAINNET_CHAIN_ID;
        W.$("chainId").readOnly = true;
        W.$("networkTag").textContent = `Mainnet chain ${W.MAINNET_CHAIN_ID}`;
      }
    } catch (e) {
      W.log(e.message, "err");
    }
  };

  W.$("rpc").value = W.sdkOrigin() || "http://127.0.0.1:8083";
  W.$("rpc").addEventListener("change", W.warnRpcOriginMismatch);
  W.$("rpc").addEventListener("blur", W.warnRpcOriginMismatch);

  W.$("btnTestnet").onclick = () => {
    if (!W.state.hip25Dev) {
      W.log("Testnet seed fill is disabled on mainnet nodes", "err");
      return;
    }
    W.$("address").value = W.TESTNET.address;
    W.$("fee").value = W.TESTNET.fee;
    W.$("chainId").value = "1";
    W.$("rpc").value = W.sdkOrigin();
    if (!W.$("prikey").value.trim()) W.$("prikey").value = W.TESTNET.password;
    W.resolveSigningAddress().then(() => W.loadPortfolio()).catch((e) => W.log(e.message, "err"));
    W.log("Filled testnet seed — loading portfolio…");
  };

  W.$("prikey").addEventListener("input", () => W.resolveSigningAddress());
  W.$("address").addEventListener("input", W.updateAddressMismatchWarning);

  W.$("btnLoad").onclick = () => W.loadPortfolio().catch((e) => W.log(e.message, "err"));
  W.$("btnRefresh").onclick = () => W.loadPortfolio().catch((e) => W.log(e.message, "err"));
  W.$("btnStake").onclick = () => W.submitStakeAction(34).catch((e) => W.log(e.message, "err"));
  W.$("btnUnstake").onclick = () => W.submitStakeAction(35).catch((e) => W.log(e.message, "err"));

  W.$("btnSelAll").onclick = () => {
    W.state.diamonds.forEach((d) => {
      if (W.diamondSelectable(d)) W.state.selected.add(d.literal);
    });
    W.renderDiamonds();
    W.scheduleLoanRefresh();
  };
  W.$("btnSelNone").onclick = () => {
    W.state.selected.clear();
    W.renderDiamonds();
    W.$("mortgageCostHint").textContent = "";
  };

  W.$("btnNewLendId").onclick = () => {
    W.$("lendId").value = W.randomLendIdHex();
    W.updateActionButtons();
  };

  W.$("btnCalcLoan").onclick = () =>
    W.prepareMortgageOpen({ quiet: false }).catch((e) => W.log(e.message, "err"));

  W.$("btnMortgageQuote").onclick = async () => {
    const id = W.$("lendId").value.trim();
    if (!id) return W.log("Lending ID required", "err");
    try {
      const q = await W.apiGet("query/mortgage/contract", {
        id,
        redeemer: W.$("address").value.trim(),
        height: W.state.height ? String(W.state.height) : "0",
      });
      W.$("ransomAmount").value = q.min_ransom || "";
      W.$("mortgageMeta").textContent = `Redemption phase: ${q.redeem_phase || "—"} · min ransom: ${q.min_ransom || "—"}`;
      W.log(`Quote: ${q.min_ransom} (${q.redeem_phase})`, "ok");
    } catch (e) {
      W.log(e.message, "err");
    }
  };

  W.$("btnMortgageOpen").onclick = () => W.submitMortgage(15).catch((e) => W.log(e.message, "err"));
  W.$("btnMortgageRedeem").onclick = () => W.submitMortgage(16).catch((e) => W.log(e.message, "err"));

  W.bootstrap();
})(window.Hip25Wallet);