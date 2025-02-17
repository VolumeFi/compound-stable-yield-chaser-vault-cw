# Compound Stable Yield Chaser Vault CosmWasm smart contract

## Overview

This repository contains the implementation of the Compound Stable Yield Chaser Vault CW. The contract is designed to manage and execute various financial operations related to yield chasing on the Compound protocol.

## Installation & Setup
1. Clone the repository:
```
git clone https://github.com/your-repo/compound-stable-yield-chaser-vault-cw.git
cd compound-stable-yield-chaser-vault-cw
```
2. Install Rust and Cargo:
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```
3. Build the contract:
```
cargo build --release
```
4. Run tests:
```
cargo test
```

## Usage
### Deploying the Contract
To deploy the compiled CosmWasm contract:
```
wasmd tx wasm store artifacts/compound_stable_yield_chaser_vault_cw.wasm --from wallet --gas auto --gas-adjustment 1.2
```


### Interacting with the Contract
#### Changing Asset
```
{
  "change_asset": {
    "new_asset": "paloma1xyz...",
    "swap_info": {
        "route": [
            "0x1234...",
            "0x6789...",
            "0x0abc..."
            ],
        "swap_params": [
            [
                0, 0, 0, 0, 0
            ],
            ...
        ],
        "amount": "100000000",
        "expected": "10000000000",
        "pools": [
            "0x1234..."
            "0x5678...",
            ...
            ],
    }
  }
}
```

#### Setting Bobby Address
```
{
  "set_bobby": {
    "bobby": "0x1234..."
  }
}
```

#### Release Bobby token
```
{
  "release_bobby": {
    "recipient": "0x1234...",
    "amount": "1000000",
    "nonce": "10",
  }
}
```

#### Retry Releasing Bobby token
```
{
  "release_bobby": {
    "nonce": "10",
  }
}
```


## Execute Messages

Except for `UpdateJobId`, these messages send a Job scheduler message to trigger a transaction to a specific EVM smart contract. The transaction goes through Compass-EVM and attaches the Paloma address in bytes32 at the end of the transaction payload data to verify the Job scheduler message sender in EVM smart contracts.

- **ChangeAsset**: Changes the asset being managed by the contract.
- **SetBobby**: Sets a new Bobby token address.
- **ReleaseBobby**: Releases Bobby tokens to a specified recipient.
- **ReRelease**: Retries releasing Bobby tokens if the initial release fails.
- **SetPaloma**: Sets the Paloma address in EVM smart contract.
- **UpdateCompass**: Updates the compass address in EVM smart contract.
- **UpdateRefundWallet**: Updates the refund wallet address.
- **UpdateEntranceFee**: Updates the entrance fee.
- **UpdateServiceFeeCollector**: Updates the service fee collector address.
- **UpdateServiceFee**: Updates the service fee.
- **UpdateJobId**: Updates the job ID used by the CosmWasm contract.

## Modules

- **execute**: Contains the implementation of the execution messages.
- **error**: Defines custom errors for the contract.
- **msg**: Defines the messages used by the contract.
- **state**: Manages the state of the contract.

## License

This project is licensed under the Apache 2.0 License.
