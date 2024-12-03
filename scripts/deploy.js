const hre = require("hardhat");

async function main() {
    const feeRecipient = "0xYourFeeRecipientAddress"; // Replace with the actual fee recipient address
    const ReplicaTrader = await hre.ethers.getContractFactory("ReplicaTrader");
    const replicaTrader = await ReplicaTrader.deploy(feeRecipient); // Pass the constructor argument

    await replicaTrader.deployed();
    console.log("ReplicaTrader deployed to:", replicaTrader.address);
}

main().catch((error) => {
    console.error(error);
    process.exitCode = 1;
});