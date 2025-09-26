// =============================================================================
// DWRCasts Backend Constants
// =============================================================================
// This file contains all constants used throughout the backend to enable
// easy tuning and configuration from a single location.

// =============================================================================
// CONTRACT ADDRESSES
// =============================================================================

/// DWR Casts contract address on Base mainnet
pub const DWR_CASTS_CONTRACT_ADDRESS: &str = "0x1108F177596f7A2a913ABf6C208FACEf152C3d8c";

/// Farcaster Collectible Casts contract address on Base mainnet  
pub const FARCASTER_COLLECTIBLE_CASTS_CONTRACT_ADDRESS: &str = "0xc011Ec7Ca575D4f0a2eDA595107aB104c7Af7A09";

/// Farcaster Pro OG contract address
pub const FARCASTER_PRO_OG_CONTRACT_ADDRESS: &str = "0x61886e7d61f4086ada1829880af440aa0de3fc96";

// =============================================================================
// BLOCKCHAIN CONFIGURATION
// =============================================================================

/// Base mainnet chain ID
pub const BASE_MAINNET_CHAIN_ID: u64 = 8453;

/// DWR's Farcaster ID (for cast validation)
pub const DWR_FARCASTER_FID: i64 = 3;

// =============================================================================
// EVENT TOPICS (for blockchain event watching)
// =============================================================================

/// AuctionStarted event topic
pub const AUCTION_STARTED_TOPIC: &str = "0xff806b81f0835f88057555bc17fb31912ff47d1cf9240f611693dcebb314d322";

/// AuctionSettled event topic  
pub const AUCTION_SETTLED_TOPIC: &str = "0x16702db8515cd96559fff387e936d2e1d3d73133dcc6eb4d9ca8eed1aa6e2844";

/// PresaleClaimed event topic
pub const PRESALE_CLAIMED_TOPIC: &str = "0xe0270c82313d232e67828d1d32f511c912186a68279bff9f57b2325d4840c91a";

// =============================================================================
// BLOCKCHAIN WATCHER CONFIGURATION
// =============================================================================

/// Starting block for auction watcher
pub const AUCTION_WATCHER_START_BLOCK: u64 = 33200642; // 0x1FA9A02

/// How often to fetch new blocks for auction watcher
pub const AUCTION_WATCHER_FETCH_INTERVAL_SECS: u64 = 30;

/// Initial chunk size for blockchain scanning
pub const INITIAL_CHUNK_SIZE: u64 = 500;

/// Minimum chunk size for blockchain scanning
pub const MIN_CHUNK_SIZE: u64 = 125;

/// Maximum chunk size for blockchain scanning  
pub const MAX_CHUNK_SIZE: u64 = 4000;

/// How often to refresh latest block number
pub const REFRESH_LATEST_BLOCK_EVERY_N_ITERS: usize = 10;

/// Watcher ID for auction watcher
pub const AUCTION_WATCHER_ID: &str = "auction_watcher";

// =============================================================================
// EIP712 CONFIGURATION
// =============================================================================

/// EIP712 domain name for wrapping signatures
pub const WRAP_DOMAIN_NAME: &str = "DWRCasts";

/// EIP712 domain version for wrapping signatures
pub const WRAP_DOMAIN_VERSION: &str = "1";

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
// NFT API LIMITS
// =============================================================================

/// Maximum NFTs to fetch per request for DWRCasts
pub const MAX_DWRCASTS_PER_REQUEST: u32 = 100;

/// Maximum NFTs to fetch per request for Farcaster Collectible Casts
pub const MAX_FARCASTER_COLLECTIBLES_PER_REQUEST: u32 = 300;

/// Default page size for Alchemy API requests
pub const ALCHEMY_DEFAULT_PAGE_SIZE: u32 = 100;

/// Maximum total NFTs to return to frontend
pub const MAX_NFTS_TOTAL_RETURN: u32 = 1000;

// =============================================================================
// PRESALE CONFIGURATION
// =============================================================================

/// Maximum total supply available for presale
pub const MAX_PRESALE_SUPPLY: i64 = 5300;

/// Maximum NFTs one address can purchase during presale
pub const MAX_PRESALE_NFTS_PER_ADDRESS: i64 = 250;

/// Minimum quantity for presale purchase
pub const MIN_PRESALE_QUANTITY: u32 = 1;

/// Maximum quantity for single presale purchase (same as address limit)
pub const MAX_PRESALE_QUANTITY: u32 = 250;

/// How long (in minutes) to hold designated NFTs before expiring them
pub const PRESALE_DESIGNATION_EXPIRY_MINUTES: i64 = 10;

// =============================================================================
// DATABASE CONFIGURATION
// =============================================================================

/// Default chunk size for blockchain watcher operations
pub const DEFAULT_WATCHER_CHUNK_SIZE: i64 = 500;

/// Maximum character limit for description in metadata
pub const DESCRIPTION_CHAR_LIMIT: usize = 400;

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
