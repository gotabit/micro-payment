## 1: transaction sign circuit proof

metadata raw_transaction {
    pubkey_hash: [u8]
    chan_id: u64
    nonce: u64
    amount: option<u128>
}

### circuit input
public
    pubkey_hash bytes
    raw_transaction bytes
private
    sign bytes
    pubkey bytes


### circuit contraints
    1. hash(pubkey) === pubkey_hash
    2. verify_sign(raw_transaction, sign, pubkey) === true