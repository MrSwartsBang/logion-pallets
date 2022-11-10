# pallet-logion-vault
The pallet to manage Legal Officer protected Multi-sign account.

Provides entry points for
* Wallet user to request an operation on a vault (typically, a transfer).
* Legal officer to approve the operation.

A **vault** is a multi-sign Polkadot account with specific rules:
* It has 3 signatories (one wallet user and 2 legal officers, selected by the user).
* The threshold is 2.
* The creation of an operation can only be done by the wallet user, owner of the vault.
* The approval of an operation can only be done by one legal officer.

## Use, Build and Publish
Details on how to use, build and publish can be found [here](https://github.com/logion-network/logion-shared#readme)
