
const { Connection, PublicKey } = require("@solana/web3.js");
const conn = new Connection("https://devnet.helius-rpc.com/?api-key=74040312-f5c1-4c9d-8339-1fb3d043d5e0", { commitment: "confirmed" });
const PROG = new PublicKey("h6zsnHt28NpeS94Ek3fQP1YEiu1WrpGT2pKynWZzKVX");

conn.getSignaturesForAddress(PROG, { limit: 30 }).then(async sigs => {
  const ok = sigs.filter(s => s.err === null);
  const results = [];
  const queues = [], callbacks = [];

  for (const s of ok.slice(0, 20)) {
    const tx = await conn.getTransaction(s.signature, {
      maxSupportedTransactionVersion: 0, commitment: "confirmed"
    });
    if (tx === null) continue;
    const logs = tx.meta.logMessages || [];
    const cuLog = logs.find(l => l.includes("consumed") && l.includes(PROG.toBase58()));
    const ixLog = logs.find(l =>
      l.includes("Instruction:") &&
      l.indexOf("ComputeBudget") === -1 &&
      l.indexOf("System") === -1
    );

    if (logs.some(l => l.includes("CallbackComputation"))) callbacks.push(s.blockTime);
    else if (logs.some(l => l.includes("QueueComputation"))) queues.push(s.blockTime);

    if (cuLog === undefined || ixLog === undefined) continue;
    const m = cuLog.match(/consumed (\d+)/);
    const name = ixLog.replace("Program log: Instruction: ", "").trim();
    results.push({ name, cu: parseInt(m[1]), fee: tx.meta.fee, slot: tx.slot });
  }

  console.log("=== COMPUTE UNITS PER INSTRUCTIE ===");
  console.log("Instructie           | CU       | Fee SOL  | Fee USD");
  console.log("-".repeat(60));
  results.forEach(r => {
    const usd = (r.fee/1e9*170).toFixed(5);
    console.log(r.name.padEnd(20) + " | " + r.cu.toString().padEnd(8) + " | " + (r.fee/1e9).toFixed(6) + " | $" + usd);
  });

  if (results.length > 0) {
    const avgCU  = Math.round(results.reduce((s,r) => s+r.cu, 0)/results.length);
    const avgFee = results.reduce((s,r) => s+r.fee, 0)/results.length;
    console.log("");
    console.log("Gemiddeld CU:       " + avgCU);
    console.log("Gemiddeld fee:      " + (avgFee/1e9).toFixed(6) + " SOL");
    console.log("Fee/order bij 70: $" + (avgFee/1e9*170).toFixed(5));
    console.log("1000 orders/dag:    $" + (avgFee/1e9*170*1000).toFixed(2) + "/dag");
    console.log("1M orders/dag:      $" + (avgFee/1e9*170*1000000).toFixed(0) + "/dag");
  }

  console.log("");
  console.log("=== MPC LATENCY ===");
  if (queues.length > 0 && callbacks.length > 0) {
    const n = Math.min(queues.length, callbacks.length);
    const lats = [];
    for (let i = 0; i < n; i++) lats.push(Math.abs(callbacks[i] - queues[i]));
    const avg = Math.round(lats.reduce((a,b) => a+b)/lats.length);
    console.log("Min latency:  " + Math.min(...lats) + "s");
    console.log("Max latency:  " + Math.max(...lats) + "s");
    console.log("Gem latency:  " + avg + "s");
    console.log("Metingen:     " + n);
    console.log("");
    console.log("Inschatting mainnet: vergelijkbaar of sneller");
    console.log("Kosten mainnet:      +15-30% door echte SOL prijs");
  }
});
