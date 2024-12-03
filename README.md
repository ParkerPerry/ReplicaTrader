# Replica Trader

A hybrid blockchain trading system integrating on-chain subscription and settings management with off-chain trade execution using Binance API.

## Features

- On-chain subscription management with multi-tier support.
- User-specific risk settings (stop-loss, take-profit, slippage).
- Event-based trade triggering for off-chain execution.
- Backend integration with Binance API for trade execution.

## Project Structure

replica-trader/
├── contracts/                # Smart contracts
├── backend/                  # Backend for event listening and trade execution
├── config/                   # Configuration files
├── database/                 # Database migration scripts
├── tests/                    # Unit tests
├── scripts/                  # Deployment and interaction scripts
├── hardhat.config.js         # Hardhat configuration
├── package.json              # Node.js package dependencies
├── README.md                 # Project documentation


## Prerequisites

1. Node.js and npm installed.
2. Hardhat installed globally:
   ```bash
   npm install -g hardhat
3.	Infura account for connecting to Ethereum testnets.
4.	Binance API keys for trade execution.


Setup

	1.	Clone the repository:

git clone https://github.com/your-repo/replica-trader.git
cd replica-trader

	2.	Install dependencies:

npm install

	3.	Create a .env file in the config/ directory:

INFURA_PROJECT_ID=your-infura-project-id
PRIVATE_KEY=your-wallet-private-key
BINANCE_API_KEY=your-binance-api-key


Deployment

	1.	Compile the smart contracts:

npx hardhat compile

	2.	Deploy the contract to Goerli testnet:

npm run deploy


Interaction

	1.	Interact with the deployed contract:

npm run interact



Backend

The backend monitors events emitted by the smart contract and executes trades using Binance API.
	1.	Navigate to the backend/ directory:

cd backend

	2.	Run the backend:
cargo run



Testing

	Run the unit tests:

npx hardhat test


