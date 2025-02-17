# Compound Stable Yield Chaser Vault CosmWasm smart contract

## Overview

This repository contains the implementation of the Compound Stable Yield Chaser Vault CW. The contract is designed to manage and execute various financial operations related to yield chasing on the Compound protocol.

## Execute Messages

These messages except for `UpdateJobId` send a Job scheduler message to trigger transaction to specific EVM smart contract. The transaction goes through Compass-EVM and it attatches paloma address in bytes32 at the last piece of transaction payload data to check Job scheduler message sender in EVM smart contracts.

- **ChangeAsset**: Changes the asset being managed by the contract.
- **SetBobby**: Sets a new Bobby address.
- **ReleaseBobby**: Releases Bobby token to a specified recipient.
- **ReRelease**: Retry releasing Bobby tokens if the initial release fails.
- **SetPaloma**: Sets the Paloma address in EVM smart contract.
- **UpdateCompass**: Updates the compass address in EVM smart contract.
- **UpdateRefundWallet**: Updates the refund wallet address.
- **UpdateEntranceFee**: Updates the entrance fee.
- **UpdateServiceFeeCollector**: Updates the service fee collector address.
- **UpdateServiceFee**: Updates the service fee.
- **UpdateJobId**: Updates the job ID in the CosmWasm contract.

## Modules

- **execute**: Contains the implementation of the execution messages.
- **error**: Defines custom errors for the contract.
- **msg**: Defines the messages used by the contract.
- **state**: Manages the state of the contract.

## License

This project is licensed under the Apache 2.0 License.
