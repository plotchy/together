// SPDX-License-Identifier: MIT
pragma solidity 0.8.30;

import {OwnableUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/access/OwnableUpgradeable.sol";
import {UUPSUpgradeable} from "openzeppelin-contracts-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import {Strings} from "openzeppelin-contracts/contracts/utils/Strings.sol";
import {ECDSA} from "openzeppelin-contracts/contracts/utils/cryptography/ECDSA.sol";
import {EIP712Upgradeable} from "openzeppelin-contracts-upgradeable/contracts/utils/cryptography/EIP712Upgradeable.sol";

/**
 * @title Together
 * @author plotchy
 * @notice Together
 */
contract Together is OwnableUpgradeable, UUPSUpgradeable, EIP712Upgradeable {

    error Unauthorized(); // Caller is unauthorized for this operation
    error InvalidInput(); // Input parameters are malformed or invalid
    error DeadlineExpired(); // Signature deadline has passed
    error NonceAlreadyUsed(); // Nonce has already been used

    event SignerAllowed(address indexed account);
    event SignerDenied(address indexed account);
    event TogetherEvent(address indexed onBehalfOf, address indexed togetherWith, uint256 indexed timestamp);
    event UserTogetherCountUpdated(address indexed account, uint256 indexed togetherCount);

    struct TogetherEventData {
        address onBehalfOf;
        address togetherWith;
        uint256 timestamp;
    }

    struct TogetherHalf {
        address togetherWith;
        uint256 timestamp;
    }

    /**
     * @notice Offchain authorizer signature data
     * @param nonce Replay protection nonce. Random 32 byte value.
     * @param deadline Signature expiration timestamp
     * @param signature EIP-712 signature bytes
     */
    struct AuthData {
        bytes32 nonce;
        uint256 deadline;
        bytes signature;
    }

    /// @notice Mapping of address to together auth status
    mapping(address account => bool authorized) public signers;

    /// @notice Mapping of address to together status
    mapping(address account => mapping(address togetherWith => uint256 timestamp)) public togetherStatus;

    /// @notice Mapping of address to together count
    mapping(address account => uint256 togetherCount) public togetherCount;

    /// @notice Mapping of address to together list
    mapping(address account => TogetherHalf[] togetherList) public togetherList;

    /// @notice Total number of togethers
    uint public totalTogether;

    /// @notice Mapping of nonce to used status
    mapping(bytes32 nonce => bool used) public authNoncesUsed;

    /// @notice EIP-712 type hash for wrap data
    bytes32 public constant TOGETHER_TYPEHASH = keccak256(
        "TogetherData(address onBehalfOf,address togetherWith,uint256 timestamp)"
    );

    constructor() {
        _disableInitializers();
    }

    function initialize(address _owner) external initializer {
        __Ownable_init(_owner);
    }

    function together(address onBehalfOf, address togetherWith, uint256 timestamp, AuthData calldata authData) external {
        /*

        validate signature is from a signer and the sig matches the onBehalfOf, togetherWith, and timestamp

        increment each user's togetherCount

        increment totalTogethers
        add to each user's togetherList
        add to each user's togetherStatus


        */
        if (!signers[msg.sender]) revert Unauthorized();
        
        // Verify signature first
        _verifyTogetherSignature(onBehalfOf, togetherWith, timestamp, authData);
        
        togetherCount[onBehalfOf]++;
        togetherCount[togetherWith]++;
        togetherList[onBehalfOf].push(TogetherHalf(togetherWith, timestamp));
        togetherList[togetherWith].push(TogetherHalf(onBehalfOf, timestamp));
        togetherStatus[onBehalfOf][togetherWith] = timestamp;
        togetherStatus[togetherWith][onBehalfOf] = timestamp;
        totalTogether++;
        
        emit TogetherEvent(onBehalfOf, togetherWith, timestamp);
        emit UserTogetherCountUpdated(onBehalfOf, togetherCount[onBehalfOf]);
        emit UserTogetherCountUpdated(togetherWith, togetherCount[togetherWith]);
    }

    function allowSigner(address account) external onlyOwner {
        signers[account] = true;
        emit SignerAllowed(account);
    }

    function denySigner(address account) external onlyOwner {
        signers[account] = false;
        emit SignerDenied(account);
    }

    function _authorizeUpgrade(address newImplementation) internal override onlyOwner {}

    /**
     * @notice Verify together signature
     * @param onBehalfOf Recipient address
     * @param togetherWith Together with address
     * @param timestamp Timestamp
     * @param authData Authorization data with signature
     * @dev Internal function to verify EIP-712 together signatures
     */
    function _verifyTogetherSignature(address onBehalfOf, address togetherWith, uint256 timestamp, AuthData calldata authData) internal {
        // Check deadline
        if (authData.deadline < block.timestamp) {
            revert DeadlineExpired();
        }
        
        // Check nonce hasn't been used
        if (authNoncesUsed[authData.nonce]) {
            revert NonceAlreadyUsed();
        }
        
        // Mark nonce as used
        authNoncesUsed[authData.nonce] = true;
        
        // Create the struct hash for together
        bytes32 structHash = keccak256(abi.encode(
            TOGETHER_TYPEHASH,
            onBehalfOf,
            togetherWith,
            timestamp,
            authData.nonce,
            authData.deadline
        ));
        
        // Get the typed data hash
        bytes32 hash = _hashTypedDataV4(structHash);
        
        // Recover signer and verify they're authorized
        address signer = ECDSA.recover(hash, authData.signature);
        if (!signers[signer]) {
            revert Unauthorized();
        }
    }
}
