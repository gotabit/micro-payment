# Micro payment channel interface 

enum PaymentType {
    FaceValue,
    AnyValue,
}

## Common interface
```rust
 fn contract_call(contract_addr: String, raw_msg: bytes, sign: bytes)
 fn contract_query(contrat_addr: String, query_msg: bytes) -> Response<T>
```


## Sender interface

```rust
 fn build_payment(key_hash: bytes, chann_id: u64, amount: Option<u128>)
 // RPC fn combine_payment(tiny_payemnt: vec<(raw_transaction, proof)>, payment_type: PaymentType) -> (raw_transaction, tx_proof)
```


## Recipient interface
```rust
 RPC fn recv_payment(payment_proof: bytes, raw_transaction: bytes, id: bytes, payment_type: PaymentType) -> bool
```