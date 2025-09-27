// =============================================================================
// Together Backend Constants
// =============================================================================
// This file contains all constants used throughout the backend to enable
// easy tuning and configuration from a single location.

// =============================================================================
// CONTRACT ADDRESSES
// =============================================================================

/// Together contract address on Worldchain mainnet
pub const TOGETHER_CONTRACT_ADDRESS: &str = "0x0053E5F890d5cE67048C86eCCf6051A92Ab34b4b";

// =============================================================================
// BLOCKCHAIN CONFIGURATION
// =============================================================================

/// Worldchain mainnet chain ID
pub const WORLDCHAIN_MAINNET_CHAIN_ID: u64 = 480;


// =============================================================================
// EVENT TOPICS (for blockchain event watching)
// =============================================================================
/// EIP712DomainChanged() event topic
pub const EIP712_DOMAIN_CHANGED_TOPIC: &str = "0x0a6387c9ea3628b88a633bb4f3b151770f70085117a15f9bf3787cda53f13d31";

/// Initialized(uint64) event topic
pub const INITIALIZED_TOPIC: &str = "0xc7f505b2f371ae2175ee4913f4499e1f2633a7b5936321eed1cdaeb6115181d2";

/// OwnershipTransferred(address,address) event topic
pub const OWNERSHIP_TRANSFERRED_TOPIC: &str = "0x8be0079c531659141344cd1fd0a4f28419497f9722a3daafe3b4186f6b6457e0";

/// SignerAllowed(address) event topic
pub const SIGNER_ALLOWED_TOPIC: &str = "0x2188e0ab4ed4b0fc2d8abb578afcaeae3688a524211cfe172e2d0079ad9bcbe7";

/// SignerDenied(address) event topic
pub const SIGNER_DENIED_TOPIC: &str = "0xca9c03a81524e5ee920d2d3b97297404e45d8ab8f5bd22ebeb1bbe079a427df1";

/// TogetherEvent(address,address,uint256) event topic
pub const TOGETHER_EVENT_TOPIC: &str = "0xd996368b8a5e10ee4a90327ea0189598b821abd2e031403b29ad5a4f90f99ca4";

/// Upgraded(address) event topic
pub const UPGRADED_TOPIC: &str = "0xbc7cd75a20ee27fd9adebab32041f755214dbc6bffa90cc0225b39da2e5c2d3b";

/// UserTogetherCountUpdated(address,uint256) event topic
pub const USER_TOGETHER_COUNT_UPDATED_TOPIC: &str = "0x074abe9c54a849285ed05fef2b25d336a525cedfbffc74362d1d4742465c8261";

// =============================================================================
// BLOCKCHAIN WATCHER CONFIGURATION
// =============================================================================

/// Starting block for attestation watcher
pub const ATTESTATION_WATCHER_START_BLOCK: u64 = 19791116; // 0x12DFD0C

/// How often to fetch new blocks for auction watcher
pub const ATTESTATION_WATCHER_FETCH_INTERVAL_SECS: u64 = 30;

/// Initial chunk size for blockchain scanning
pub const INITIAL_CHUNK_SIZE: u64 = 500;

/// Minimum chunk size for blockchain scanning
pub const MIN_CHUNK_SIZE: u64 = 125;

/// Maximum chunk size for blockchain scanning  
pub const MAX_CHUNK_SIZE: u64 = 4000;

/// How often to refresh latest block number
pub const REFRESH_LATEST_BLOCK_EVERY_N_ITERS: usize = 2;

/// Watcher ID for auction watcher
pub const ATTESTATION_WATCHER_ID: &str = "attestation_watcher";

// =============================================================================
// EIP712 CONFIGURATION
// =============================================================================

/// EIP712 domain name for together signatures
pub const TOGETHER_DOMAIN_NAME: &str = "Together";

/// EIP712 domain version for together signatures
pub const TOGETHER_DOMAIN_VERSION: &str = "1";

/// Signature deadline duration in minutes
pub const SIGNATURE_DEADLINE_MINUTES: i64 = 3;

// =============================================================================
// RATE LIMITING
// =============================================================================

/// Maximum Alchemy API requests per minute per address
pub const ALCHEMY_RATE_LIMIT_PER_MINUTE: u32 = 10;

/// Rate limit window duration in seconds
pub const RATE_LIMIT_WINDOW_SECONDS: u64 = 60;

// =============================================================================
// DATABASE CONFIGURATION
// =============================================================================

/// Default chunk size for blockchain watcher operations
pub const DEFAULT_WATCHER_CHUNK_SIZE: i64 = 500;

// =============================================================================
// ADDRESS VALIDATION
// =============================================================================

/// Expected length of Ethereum address (including 0x prefix)
pub const ETHEREUM_ADDRESS_LENGTH: usize = 42;

/// Ethereum address prefix
pub const ETHEREUM_ADDRESS_PREFIX: &str = "0x";

/// Length of cast hash (including 0x prefix)
pub const CAST_HASH_LENGTH: usize = 66;

// =============================================================================
// SERVER CONFIGURATION
// =============================================================================

/// Default server port if not specified in environment
pub const DEFAULT_SERVER_PORT: u16 = 3000;

// =============================================================================
// HELPER FUNCTIONS FOR VALIDATION
// =============================================================================

/// Validates if a string is a valid Ethereum address format
pub fn is_valid_ethereum_address(address: &str) -> bool {
    address.starts_with(ETHEREUM_ADDRESS_PREFIX) && address.len() == ETHEREUM_ADDRESS_LENGTH
}

/// Validates if a string is a valid cast hash format
pub fn is_valid_cast_hash(hash: &str) -> bool {
    hash.starts_with(ETHEREUM_ADDRESS_PREFIX) && hash.len() == CAST_HASH_LENGTH
}
