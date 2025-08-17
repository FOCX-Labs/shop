# The shop module of FOCX

The solana smart contract implements the core shop module of FOCX, which is a Fully On-Chain E-commerce Protocol without backend and off-chain indexer. 

The shop contract allows users to buy and sell products/services, with functionalities of merchant registeration, merchant security deposit, product searching, product CRUD, order placement & payment & shipping & confirmation. 

It is also integrated with [Insurance Fund Vault](https://github.com/FOCX-Labs/vault) to add platform fee to the vault so that users can share reward from the protocol. 

We also innovatively implemented the keywords & price & sales on-chain index, which makes fully on-chain product searching possible.

## Quick start
### Requirement
 - anchor 0.31.1
### Build
```bash
anchor build
```
### Deploy
```bash
anchor deploy
```
