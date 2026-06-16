/* Portfolio load and HACD list */
(function (W) {
  let loadRetryTimer = null;

  W.schedulePortfolioRetry = () => {
    if (loadRetryTimer) return;
    loadRetryTimer = setTimeout(() => {
      loadRetryTimer = null;
      W.loadPortfolio({ retry: true }).catch((e) => W.log(e.message, "err"));
    }, 2000);
  };

  W.clearPortfolioRetry = () => {
    if (!loadRetryTimer) return;
    clearTimeout(loadRetryTimer);
    loadRetryTimer = null;
  };

  W.selectedAvailableDiamonds = () => {
    const byName = Object.fromEntries(W.state.diamonds.map((d) => [d.literal, d]));
    return [...W.state.selected].filter((n) => byName[n]?.status === "Available");
  };

  W.diamondSelectable = (d) => {
    if (d.status === "Available") return true;
    if (d.status === "Staked" && W.state.height >= (d.min_unstake_height || 0)) return true;
    if (d.status === "Mortgaged") return true;
    return false;
  };

  W.restoreSelection = (prevSelected) => {
    W.state.selected.clear();
    if (!prevSelected.size) return;
    const byName = Object.fromEntries(W.state.diamonds.map((d) => [d.literal, d]));
    for (const lit of prevSelected) {
      const d = byName[lit];
      if (d && W.diamondSelectable(d)) W.state.selected.add(lit);
    }
  };

  W.updateActionButtons = () => {
    const sel = [...W.state.selected];
    const byName = Object.fromEntries(W.state.diamonds.map((d) => [d.literal, d]));
    const canStake = sel.some((n) => byName[n]?.status === "Available");
    const canUnstake = sel.some(
      (n) => byName[n]?.status === "Staked" && W.state.height >= (byName[n].min_unstake_height || 0)
    );
    const canMortgageOpen = sel.some((n) => byName[n]?.status === "Available");
    const wasmOk = !!W.state.wasm;
    W.$("btnStake").disabled = !wasmOk || !canStake;
    W.$("btnUnstake").disabled = !wasmOk || !canUnstake;
    W.$("btnMortgageOpen").disabled = !wasmOk || !canMortgageOpen;
    W.$("btnMortgageRedeem").disabled = !wasmOk || !W.$("lendId").value.trim();
  };

  W.renderDiamonds = () => {
    const box = W.$("diamonds");
    box.innerHTML = "";
    if (!W.state.diamonds.length) {
      box.innerHTML = '<div class="empty">No HACD found for this address.</div>';
      return;
    }
    for (const d of W.state.diamonds) {
      const row = document.createElement("div");
      row.className = "diamond-row" + (W.state.selected.has(d.literal) ? " selected" : "");
      const canStake = d.status === "Available";
      const canUnstake = d.status === "Staked" && W.state.height >= d.min_unstake_height;
      const isMortgaged = d.status === "Mortgaged";
      const cb = document.createElement("input");
      cb.type = "checkbox";
      cb.checked = W.state.selected.has(d.literal);
      cb.disabled = !W.diamondSelectable(d);
      cb.onchange = () => {
        if (cb.checked) {
          W.state.selected.add(d.literal);
          if (isMortgaged && d.lending_id) {
            W.$("lendId").value = d.lending_id;
            W.$("btnMortgageQuote").click();
          }
        } else W.state.selected.delete(d.literal);
        W.updateActionButtons();
        row.classList.toggle("selected", cb.checked);
        W.scheduleLoanRefresh();
      };
      const info = document.createElement("div");
      const lit = document.createElement("div");
      lit.className = "literal";
      lit.textContent = d.literal;
      const meta = document.createElement("div");
      meta.className = "meta";
      let metaTxt = `reward ${d.accrued_reward || "0"} · min unstake block ${d.min_unstake_height ?? "—"}`;
      if (d.lending_id) metaTxt += ` · mortgage ${d.lending_id.slice(0, 8)}…`;
      meta.textContent = metaTxt;
      info.append(lit, meta);
      const badge = document.createElement("span");
      badge.className = `badge badge-${d.status}`;
      badge.textContent = d.status;
      row.append(cb, info, badge, document.createElement("span"));
      box.appendChild(row);
    }
    W.updateActionButtons();
  };

  W.loadPortfolio = async (opts = {}) => {
    const addr = W.$("address").value.trim();
    if (!addr) throw new Error("Address required");
    const prevSelected = new Set(W.state.selected);
    if (!opts.retry) W.log("Loading portfolio…");

    const latest = await W.apiGet("query/latest");
    W.state.height = latest.height;
    W.$("sHeight").textContent = latest.height;
    W.state.hip25Dev = !!latest.hip25_dev;
    if (latest.chain_id !== undefined) {
      const nodeChain = String(latest.chain_id);
      const cur = W.$("chainId").value.trim();
      if (!cur || cur !== nodeChain) {
        W.$("chainId").value = nodeChain;
        if (cur && cur !== nodeChain) W.log(`Chain ID synced to node (${nodeChain})`, "err");
      }
      W.$("chainId").readOnly = !latest.hip25_dev;
      W.$("networkTag").textContent = latest.hip25_dev ? "HIP-25 testnet" : `Mainnet chain ${W.MAINNET_CHAIN_ID}`;
    }

    const global = await W.apiGet("query/staking/global");
    W.$("sPool").textContent = global.reward_pool_pending_zhu ?? "—";
    try {
      const mg = await W.apiGet("query/mortgage/global");
      W.state.mortgageGlobal = mg;
      W.$("sMortgageIoU").textContent = mg.outstanding_ioo_zhu ?? "—";
      W.$("sMortgageApr").textContent = mg.apr_bps ? `${(mg.apr_bps / 100).toFixed(1)}%` : "—";
      const max = mg.owner_index_max ?? 64;
      W.$("mortgageIndexHint").textContent = `Up to ${max} active mortgage contracts per address (indexed in portfolio RPC).`;
    } catch (_) {
      W.state.mortgageGlobal = null;
      W.$("sMortgageIoU").textContent = "—";
      W.$("sMortgageApr").textContent = "—";
    }

    const bal = await W.apiGet("query/balance", { address: addr, diamonds: "true" });
    const entry = bal.list?.[0] || {};
    W.$("sHac").textContent = entry.hacash || "0";

    const summary = await W.apiGet("query/staking/summary", { address: addr });
    W.$("sStaked").textContent = summary.staked_count ?? 0;
    W.$("sCooldown").textContent = summary.cooldown_count ?? 0;
    W.$("sAccrued").textContent = summary.total_accrued_reward || "0";

    const contracts = await W.loadMortgagePortfolio(addr);
    const mortgagedMap = {};
    for (const c of contracts) {
      for (const d of c.diamonds || []) mortgagedMap[d] = c.lending_id;
    }
    const names = [...new Set([...W.splitDiamonds(entry.diamonds || ""), ...Object.keys(mortgagedMap)])];
    if (!names.length && W.state.height < 1) {
      W.log("Waiting for block 1 (testnet seed loads with first block)…");
      W.schedulePortfolioRetry();
      return;
    }
    W.clearPortfolioRetry();
    if (!names.length) {
      W.state.diamonds = [];
      W.state.selected.clear();
      W.renderDiamonds();
      W.log(`No HACD on ${addr} — click "Fill HIP-25 testnet seed" then Load portfolio`, "err");
      return;
    }

    W.state.diamonds = names.map((literal) => ({
      literal,
      status: "Loading",
      accrued_reward: "0",
      min_unstake_height: 0,
      lending_id: mortgagedMap[literal] || "",
    }));
    W.renderDiamonds();
    await Promise.all(
      names.map(async (literal, idx) => {
        try {
          const st = await W.apiGet("query/staking/status", { diamond: literal });
          W.state.diamonds[idx] = {
            literal: st.literal || literal,
            status: st.status || "unknown",
            accrued_reward: st.accrued_reward || "0",
            min_unstake_height: st.min_unstake_height || 0,
            lending_id: mortgagedMap[literal] || "",
          };
        } catch (e) {
          W.state.diamonds[idx] = {
            literal,
            status: mortgagedMap[literal] ? "Mortgaged" : "unknown",
            accrued_reward: "0",
            min_unstake_height: 0,
            lending_id: mortgagedMap[literal] || "",
          };
        }
      })
    );
    W.state.diamonds.sort((a, b) => a.literal.localeCompare(b.literal));
    W.restoreSelection(prevSelected);
    W.renderDiamonds();
    W.log(`Loaded ${names.length} HACD`, "ok");
    if (W.selectedAvailableDiamonds().length) {
      W.prepareMortgageOpen({ quiet: true }).catch((e) => W.log(e.message, "err"));
    }
  };

  W.submitStakeAction = async (kind) => {
    if (!W.state.wasm) throw new Error("WASM SDK required — build with scripts/build_wallet_sdk.ps1");
    const pass = W.$("prikey").value.trim();
    if (!pass) throw new Error("Password required for local signing");
    const fee = W.$("fee").value.trim();
    const chainIdStr = W.$("chainId").value.trim();
    if (!W.state.hip25Dev && chainIdStr !== W.MAINNET_CHAIN_ID) {
      throw new Error(`Mainnet wallet requires chain ID ${W.MAINNET_CHAIN_ID}`);
    }
    const chainId = BigInt(chainIdStr || W.MAINNET_CHAIN_ID);
    const sel = [...W.state.selected];
    if (!sel.length) throw new Error("Select at least one HACD");
    const diamonds = sel.join(",");
    const stake = kind === 34;
    const fn = stake ? W.state.wasm.hacd_stake : W.state.wasm.hacd_unstake;
    const ts = BigInt(W.txTimestamp());
    W.log(`[WASM] ${stake ? "hacd_stake" : "hacd_unstake"} for ${diamonds} (ts=${ts})…`);
    const raw = fn(chainId, pass, diamonds, fee, ts);
    const tx = W.parseSdkJson(raw);
    W.log(`Signed locally ${tx.tx_hash}`);
    const submitted = await W.apiPost("submit/transaction", W.hexToBytes(tx.tx_body));
    W.$("prikey").value = "";
    W.log(`Submitted ${submitted.hash}`, "ok");
    setTimeout(W.loadPortfolio, 3000);
  };
})(window.Hip25Wallet);