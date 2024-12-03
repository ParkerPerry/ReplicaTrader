const { ethers } = require("hardhat");

async function main() {
    const contractAddress = "0xYourContractAddressHere"; // Replace with deployed contract address
    const ReplicaTrader = await ethers.getContractFactory("ReplicaTrader");
    const replicaTrader = await ReplicaTrader.attach(contractAddress);

    // Example: Subscribe with ETH
    const subscriptionFee = ethers.utils.parseEther("0.1"); // Match the fee set in the contract
    const tx = await replicaTrader.subscribe(1, { value: subscriptionFee });
    await tx.wait();
    console.log("Subscribed successfully with ETH!");

    // Example: Subscribe with a token
    const tokenAddress = "0xYourTokenAddressHere"; // Replace with supported token address
    const tokenTx = await replicaTrader.subscribeWithToken(tokenAddress, 1); // Tier 1
    await tokenTx.wait();
    console.log("Subscribed successfully with Token!");

    // Example: Set user settings
    const stopLoss = 500; // Example: 5% stop-loss
    const takeProfit = 1000; // Example: 10% take-profit
    const slippage = 50; // Example: 0.5% slippage
    const minLiquidity = 10; // Example: 10 units of liquidity required
    const symbol = "BTCUSDT"; // Example trading pair symbol
    const settingsTx = await replicaTrader.setUserSettings(stopLoss, takeProfit, slippage, minLiquidity, symbol);
    await settingsTx.wait();
    console.log("User settings updated!");

    // Example: Trigger a trade
    const amount = 1; // Example: 1 unit of the asset
    const price = 20000; // Example: $20,000 per unit
    const tradeTx = await replicaTrader.triggerTrade(amount, price);
    await tradeTx.wait();
    console.log("Trade triggered!");

    // Example: Retrieve trade history
    const tradeHistory = await replicaTrader.getTradeHistory("0xYourUserAddressHere");
    console.log("Trade History:", tradeHistory);

    // Example: Retrieve contract fee recipient
    const feeRecipient = await replicaTrader.feeRecipient();
    console.log("Fee recipient address:", feeRecipient);

    // Example: Check subscription status
    const isSubscribed = await replicaTrader.isSubscribed("0xYourUserAddressHere");
    console.log("Subscription active:", isSubscribed);
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});