// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

contract ReplicaTrader is AccessControl, ReentrancyGuard {
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");

    struct Subscription {
        uint256 expiry;
        uint256 tier;
    }

    struct UserSettings {
        uint256 stopLoss;    // Basis points
        uint256 takeProfit;  // Basis points
        uint256 slippage;    // Basis points
        uint256 minLiquidity; // Minimum liquidity required for trade execution
        string symbol;       // Trading pair symbol (e.g., "BTCUSDT")
    }

    struct TradeRecord {
        uint256 tradeId;
        uint256 amount;
        uint256 price;
        uint256 timestamp;
        string symbol;
    }

    mapping(address => Subscription) public subscriptions;
    mapping(address => UserSettings) public userSettings;
    mapping(address => TradeRecord[]) public tradeHistory;
    mapping(address => uint256) public lastTradeTimestamp;

    mapping(uint256 => uint256) public subscriptionTiers; // Tier ID -> Fee in ETH
    mapping(uint256 => uint256) public subscriptionDurations; // Tier ID -> Duration
    mapping(address => uint256) public supportedTokens; // Token address -> Token fee multiplier (e.g., USDT = 1e18 for parity with ETH)

    uint256 public platformFeePercentage = 2; // Platform fee in basis points (2%)
    address public feeRecipient;

    bool public isPaused;

    event Subscribed(address indexed user, uint256 tier, uint256 expiry);
    event SettingsUpdated(
        address indexed user,
        uint256 stopLoss,
        uint256 takeProfit,
        uint256 slippage,
        uint256 minLiquidity,
        string symbol
    );
    event TradeTriggered(
        address indexed user,
        uint256 amount,
        uint256 price,
        string symbol,
        uint256 minLiquidity,
        uint256 timestamp
    );
    event TradeFailed(address indexed user, string reason);
    event EmergencyPause(bool status);
    event PlatformFeeRecipientUpdated(address indexed newRecipient);
    event SubscriptionTierUpdated(uint256 indexed tier, uint256 fee, uint256 duration);
    event SupportedTokenAdded(address indexed token, uint256 multiplier);

    constructor(address _feeRecipient) {
        _setupRole(DEFAULT_ADMIN_ROLE, msg.sender);
        feeRecipient = _feeRecipient;

        // Initialize default subscription tiers
        subscriptionTiers[1] = 100 * 10**18; // 100 ETH (in Wei)
        subscriptionDurations[1] = 30 days; // 30 days
    }

    // ------------------------
    // Subscription Management
    // ------------------------

    modifier whenNotPaused() {
        require(!isPaused, "Contract is paused");
        _;
    }

    function subscribe(uint256 tier) external payable nonReentrant whenNotPaused {
        uint256 fee = subscriptionTiers[tier];
        uint256 duration = subscriptionDurations[tier];
        require(fee > 0 && duration > 0, "Invalid subscription tier");
        require(msg.value == fee, "Incorrect fee");

        subscriptions[msg.sender].expiry = block.timestamp + duration;
        subscriptions[msg.sender].tier = tier;

        (bool success, ) = feeRecipient.call{value: msg.value}("");
        require(success, "Fee transfer failed");

        emit Subscribed(msg.sender, tier, subscriptions[msg.sender].expiry);
    }

    function subscribeWithToken(address token, uint256 tier) external nonReentrant whenNotPaused {
        require(supportedTokens[token] > 0, "Unsupported token");
        uint256 fee = subscriptionTiers[tier] * supportedTokens[token]; // Fee adjusted for token decimals
        uint256 duration = subscriptionDurations[tier];
        require(fee > 0 && duration > 0, "Invalid subscription tier");

        IERC20(token).transferFrom(msg.sender, feeRecipient, fee);

        subscriptions[msg.sender].expiry = block.timestamp + duration;
        subscriptions[msg.sender].tier = tier;

        emit Subscribed(msg.sender, tier, subscriptions[msg.sender].expiry);
    }

    function addSupportedToken(address token, uint256 multiplier) external onlyRole(ADMIN_ROLE) {
        supportedTokens[token] = multiplier;
        emit SupportedTokenAdded(token, multiplier);
    }

    function updateSubscriptionTier(uint256 tier, uint256 fee, uint256 duration) external onlyRole(ADMIN_ROLE) {
        subscriptionTiers[tier] = fee;
        subscriptionDurations[tier] = duration;

        emit SubscriptionTierUpdated(tier, fee, duration);
    }

    function isSubscribed(address user) public view returns (bool) {
        return subscriptions[user].expiry > block.timestamp;
    }

    // ------------------------
    // User Settings
    // ------------------------

    function setUserSettings(
        uint256 stopLoss,
        uint256 takeProfit,
        uint256 slippage,
        uint256 minLiquidity,
        string memory symbol
    ) external whenNotPaused {
        require(isSubscribed(msg.sender), "Subscription expired");

        userSettings[msg.sender] = UserSettings({
            stopLoss: stopLoss,
            takeProfit: takeProfit,
            slippage: slippage,
            minLiquidity: minLiquidity,
            symbol: symbol
        });

        emit SettingsUpdated(msg.sender, stopLoss, takeProfit, slippage, minLiquidity, symbol);
    }

    // ------------------------
    // Trade Execution
    // ------------------------

    function triggerTrade(uint256 amount, uint256 price) external whenNotPaused {
        require(isSubscribed(msg.sender), "Subscription expired");
        require(block.timestamp > lastTradeTimestamp[msg.sender] + 1 minutes, "Trade cooldown active");

        UserSettings memory settings = userSettings[msg.sender];

        emit TradeTriggered(
            msg.sender,
            amount,
            price,
            settings.symbol,
            settings.minLiquidity,
            block.timestamp
        );

        tradeHistory[msg.sender].push(TradeRecord({
            tradeId: tradeHistory[msg.sender].length + 1,
            amount: amount,
            price: price,
            timestamp: block.timestamp,
            symbol: settings.symbol
        }));
    }

    function getTradeHistory(address user) external view returns (TradeRecord[] memory) {
        return tradeHistory[user];
    }

    // ------------------------
    // Platform Fee Management
    // ------------------------

    function updatePlatformFeeRecipient(address newRecipient) external onlyRole(ADMIN_ROLE) {
        feeRecipient = newRecipient;
        emit PlatformFeeRecipientUpdated(newRecipient);
    }

    // ------------------------
    // Emergency Functions
    // ------------------------

    function togglePause() external onlyRole(ADMIN_ROLE) {
        isPaused = !isPaused;
        emit EmergencyPause(isPaused);
    }
}