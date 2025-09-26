// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.30;

import {console} from "forge-std/console.sol";
import "forge-std/Script.sol";
import {Together} from "../src/Together.sol";
import {ERC1967Proxy} from "openzeppelin-contracts/contracts/proxy/ERC1967/ERC1967Proxy.sol";

/*

forge script -f $FORK_RPC_URL --private-key $PRIVATE_KEY_DEPLOYER script/Together.s.sol -vvvvv

# broadcast
forge script -f $FORK_RPC_URL --private-key $PRIVATE_KEY_DEPLOYER script/Together.s.sol -vvvvv --broadcast

# allowSigner
cast send <together_addy> "allowSigner(address)" <signer_addy> --private-key $PRIVATE_KEY_DEPLOYER --rpc-url $FORK_RPC_URL


*/

contract DeployTogether is Script {
    struct DeploymentParams {
        address deployer;
        address owner;
        address backendSigner;
    }

    struct Addresses {
        address together;
    }

    struct Contracts {
        Together together;
        ERC1967Proxy proxy;
    }

    function run() public returns (Contracts memory contracts) {
        vm.startBroadcast();
        DeploymentParams memory params = loadDeploymentParams();

        console.log("");
        console.log("========================================");
        console.log("Deploying Together contracts...");
        console.log("========================================");
        console.log("Caller:", msg.sender);
        console.log("Deployer:", params.deployer);
        console.log("Owner:", params.owner);
        console.log("Backend Signer:", params.backendSigner);
        console.log("========================================");
        console.log("");

        // deploy implementation
        Together together_impl = new Together();
        // initialize call
        bytes memory initializeData = abi.encodeWithSelector(Together.initialize.selector, params.owner);
        // this calls the `upgradeToAndCall` function through the proxy on the impl.
        ERC1967Proxy proxy = new ERC1967Proxy(address(together_impl), initializeData);
        contracts = Contracts({together: Together(address(proxy)), proxy: proxy});

        // set signer
        contracts.together.allowSigner(params.backendSigner);

        console.log("");
        console.log("========================================");
        console.log("Deployment complete!");
        console.log("========================================");
        console.log("");
        vm.stopBroadcast();

        return contracts;
    }

    function loadDeploymentParams() internal view returns (DeploymentParams memory params) {
        params.deployer = vm.envOr("DEPLOYER_ADDRESS", msg.sender);
        params.owner = vm.envOr("OWNER_ADDRESS", params.deployer);
        params.backendSigner = vm.envAddress("BACKEND_SIGNER_ADDRESS");

        return params;
    }
}