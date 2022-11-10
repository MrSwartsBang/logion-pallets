# pallet-verified-recovery
The pallet to setup a recovery by Logion Legal Officers.

Provides an entry point for Wallet User to request a protection to Legal Officer.

A Logion Protection is a specialized version of [social recovery](https://docs.rs/pallet-recovery/3.0.0/pallet_recovery/)
where
* Friends must be selected among Logion Legal Officers.
* The threshold is set to 1.
* This pallet only replaces `pallet_recovery.create_recovery()` - the remaining
of [the Life Cycle is the one of pallet_recovery](https://docs.rs/pallet-recovery/3.0.0/pallet_recovery/#recovery-life-cycle)


## Use, Build and Publish
Details on how to use, build and publish can be found [here](https://github.com/logion-network/logion-shared#readme)

