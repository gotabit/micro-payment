# Introduction
All blockchain-based transaction require a certain of time to be comfirmed due to the consensus process. It unacceptable in some high-frequency and low-latency scenarios.
Transactions at layer 2 are just like paying checks. Those who receive the check can go to the bank (on-chain) to 
collect cash at any time. The check only needs to confirm the validity of both parties without involving the bank. 

## Modules

### 1. Smart contract

* Temporarily store user transaction funds   
* Store payment channel information of both parties to the transaction
* Verify the Zero Knowledge proof of withdrawing funds

### 2. Payment channel Service
* P2P service
* One-way transaction
* Build transaction and generate Zero knowledge proof
* Verify transaction and verify Zero knowledge proof
* Transaction combination and withdrawl funds
* Close payment channel


## How it works
comming soon