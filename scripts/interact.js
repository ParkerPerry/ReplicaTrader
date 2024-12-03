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
    const settingsTx = await replicaTrader.setUserSettings(500, 1000, 50); // StopLoss, TakeProfit, Slippage
    await settingsTx.wait();
    console.log("User settings updated!");

    // Example: Trigger a trade
    const tradeTx = await replicaTrader.triggerTrade(1, 20000); // Amount, Price
    await tradeTx.wait();
    console.log("Trade triggered!");
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});