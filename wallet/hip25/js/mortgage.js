/* HIP-2 mortgage: loan prep, quotes, submit */
(function (W) {
  let loanRefreshTimer = null;

  W.updateMortgageCostHint = (principal, selCount) => {
    const hint = W.$("mortgageCostHint");
    if (!principal?.loan) {
      hint.textContent = "";
      return;
    }
    const fee = W.$("fee").value.trim() || "0";
    const bps = principal.origination_fee_bps ?? W.state.mortgageGlobal?.origination_fee_bps ?? 100;
    hint.textContent = `${principal.hacd_count || selCount} HACD → loan ${principal.loan} · origination burn (${bps / 100}%) ${principal.origination_burn || "—"} · tx fee ${fee}`;
  };

  W.prepareMortgageOpen = async (opts = {}) => {
    const sel = W.selectedAvailableDiamonds();
    if (!sel.length) {
      W.$("mortgageCostHint").textContent = "";
      if (!opts.quiet) throw new Error("Select Available HACD first");
      return null;
    }

    if (opts.preflight) {
      const signer = await W.resolveSigningAddress();
      if (!signer) throw new Error("Enter signing password");
      const portfolio = W.$("address").value.trim();
      if (portfolio && signer !== portfolio) {
        throw new Error(`Password signs as ${signer} but portfolio is ${portfolio}. Addresses must match.`);
      }
      const max = W.state.mortgageGlobal?.owner_index_max ?? 64;
      const active = W.state.mortgageContracts?.length ?? 0;
      if (active >= max) {
        throw new Error(`Mortgage index full (${active}/${max} active contracts for this address)`);
      }
    }

    const principal = await W.apiGet("query/mortgage/principal", { diamonds: sel.join(",") });
    W.$("loanAmount").value = principal.loan || "";
    W.updateMortgageCostHint(principal, sel.length);

    if (opts.preflight) {
      const signer = W.state.signAddress;
      const bal = await W.apiGet("query/balance", { address: signer });
      const hac = bal.list?.[0]?.hacash || "0";
      W.log(`Signer ${signer} balance ${hac} · need origination ${principal.origination_burn} + fee ${W.$("fee").value.trim()}`, "ok");
    } else if (!opts.quiet) {
      W.log(`Loan updated: ${principal.loan} for ${sel.join(",")}`, "ok");
    }

    return { sel, principal };
  };

  W.scheduleLoanRefresh = () => {
    if (loanRefreshTimer) clearTimeout(loanRefreshTimer);
    loanRefreshTimer = setTimeout(() => {
      loanRefreshTimer = null;
      W.prepareMortgageOpen({ quiet: true }).catch((e) => W.log(e.message, "err"));
    }, 200);
  };

  W.renderMortgageContracts = () => {
    const el = W.$("mortgageContracts");
    const list = W.state.mortgageContracts || [];
    const max = W.state.mortgageGlobal?.owner_index_max ?? 64;
    if (!list.length) {
      el.textContent = `No active mortgage contracts for this address (limit ${max}).`;
      return;
    }
    const header = list.length >= max ? `<div class="meta err" style="margin-bottom:0.35rem">Index full: ${list.length}/${max} active contracts</div>` : "";
    el.innerHTML =
      header +
      list
        .map((c) => {
          const id = c.lending_id || "";
          const dias = (c.diamonds || []).join(", ");
          return `<div style="margin:0.35rem 0;padding:0.5rem;background:var(--bg);border-radius:6px;border:1px solid var(--border)">
          <strong class="literal">${id.slice(0, 12)}…</strong> · loan ${c.loan_principal || "—"} · phase ${c.redeem_phase || "—"} · ransom ${c.min_ransom || "—"}<br/>
          HACD: ${dias || "—"}
          <button type="button" class="btn-ghost" style="margin-top:0.35rem;padding:0.25rem 0.5rem;font-size:0.75rem" data-lend="${id}" data-ransom="${c.min_ransom || ""}">Use for redeem</button>
        </div>`;
        })
        .join("");
    el.querySelectorAll("button[data-lend]").forEach((btn) => {
      btn.onclick = () => {
        W.$("lendId").value = btn.getAttribute("data-lend") || "";
        W.$("ransomAmount").value = btn.getAttribute("data-ransom") || "";
        W.updateActionButtons();
        W.log(`Selected contract ${W.$("lendId").value.slice(0, 12)}… for redeem`);
      };
    });
  };

  W.loadMortgagePortfolio = async (addr) => {
    try {
      const p = await W.apiGet("query/mortgage/portfolio", { address: addr });
      W.state.mortgageContracts = p.contracts || [];
      if (p.owner_index_max) {
        W.state.mortgageGlobal = { ...(W.state.mortgageGlobal || {}), owner_index_max: p.owner_index_max };
      }
      W.renderMortgageContracts();
      return W.state.mortgageContracts;
    } catch (e) {
      W.state.mortgageContracts = [];
      W.$("mortgageContracts").textContent = `Mortgage portfolio: ${e.message}`;
      return [];
    }
  };

  W.submitMortgage = async (kind) => {
    if (!W.state.wasm) throw new Error("WASM SDK required");
    const pass = W.$("prikey").value.trim();
    if (!pass) throw new Error("Password required");
    const fee = W.$("fee").value.trim();
    const chainId = BigInt(W.$("chainId").value.trim() || W.MAINNET_CHAIN_ID);
    const lendId = W.$("lendId").value.trim();
    const ts = BigInt(W.txTimestamp());
    let raw;
    if (kind === 15) {
      const prepared = await W.prepareMortgageOpen({ preflight: true });
      if (!prepared) throw new Error("Mortgage preparation failed");
      const sel = prepared.sel;
      const loan = W.$("loanAmount").value.trim();
      const bp = parseInt(W.$("borrowPeriod").value, 10);
      if (!lendId || !loan) throw new Error("Lending ID and loan amount required");
      if (loan !== prepared.principal.loan) {
        throw new Error(`Loan must be ${prepared.principal.loan} for selected HACD`);
      }
      raw = W.state.wasm.hacd_mortgage_open(chainId, pass, lendId, sel.join(","), loan, bp, fee, ts);
    } else {
      const ransom = W.$("ransomAmount").value.trim();
      if (!lendId || !ransom) throw new Error("Lending ID and ransom required");
      raw = W.state.wasm.hacd_mortgage_redeem(chainId, pass, lendId, ransom, fee, ts);
    }
    const tx = W.parseSdkJson(raw);
    W.log(`Signed ${tx.action} ${tx.tx_hash}`);
    const submitted = await W.apiPost("submit/transaction", W.hexToBytes(tx.tx_body));
    W.$("prikey").value = "";
    W.log(`Submitted ${submitted.hash}`, "ok");
    setTimeout(W.loadPortfolio, 3000);
  };
})(window.Hip25Wallet);