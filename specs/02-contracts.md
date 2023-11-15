# Smart contracts
This is an Cosmwasm contract which provides the following interface

* Payment channel global state
* init_payment(sender_pubkey_hash: [u8], recipients: Vec<(recipient_pubkey_hash, per_face_value, max_value)>) -> u64
* add_recipients(sender_pubkey_hash: [u8], recipients: Vec<(recipient_pubkey_hash, per_face_value, max_value)>)
* add_value(sender_pubkey_hash: [u8], chan_id: u64, value: u64)
* close_chan(sender_pubkey_hash: [u8], chan_id: u64, recipient_close_proof: Option<[u8]>)
* payment_claim(sender_pubkey_hash: [u8], chan_id: u64, payment_proof: [u8], amount: u64, nonce: u64)
* get_chan_info(sender_pubkey_hash: [u8], chan_id: u64) -> (per_face_value: u64, nonce: u64, max_value: u64)

## Contract functions

### 1. Payment channel global state
* CloseTimeout u64

### 2. init_payment(sender_pubkey_hash: [u8], recipients: Vec<(recipient_pubkey_hash, per_face_value, max_value)>) -> u64

init payment channel by given info. ps: sufficient funds need to be transferred to contract. Each address can only be init once.
| Param     |  Description  |
|----------|------:|
| sender_pubkey_hash  | sender public key hash. used for identity proof in off-chain |
| recipients |  recipients vector|
| recipient_pubkey_hash  |   recipient public key hash. used for identity proof in on-chain |
| per_face_value  |   face value per off-chain transaction  |
| max_value  |   max transaction value for specific recipient |

### 3. add_recipients(sender_pubkey_hash: [u8], recipients: Vec<(recipient_pubkey_hash, per_face_value, max_value)>)
Add multiple recipients to payment record
| Param     |  Description  |
|----------|------:|
| sender_pubkey_hash  | sender public key hash. used for identity proof in off-chain |
| recipients |  recipients vector|
| recipient_pubkey_hash  |   recipient public key hash. used for identity proof in on-chain |
| per_face_value  |   face value per off-chain transaction  |
| max_value  |   max transaction value for specific recipient |

### 4. add_value(sender_pubkey_hash: [u8], chan_id: u64, value: u64)
Add fund value for given chan id
| Param     |  Description  |
|----------|------:|
| sender_pubkey_hash  | sender public key hash. used for identity proof in off-chain |
| chan_id  | payment channel id. each path sender -> recipient called a channel |

### 5. close_chan(sender_pubkey_hash: [u8], chan_id: u64, recipient_close_proof: Option<[u8]>)
Sender request close the channel by given chan_id. if there is no proof of the recipient. close after CloseTimtout
| Param     |  Description  |
|----------|------:|
| sender_pubkey_hash  | sender public key hash. used for identity proof in off-chain |
| chan_id  | payment channel id. each path sender -> recipient called a channel |
| recipient_close_proof  | proof of close channel by recipient |


### 6. payment_claim(sender_pubkey_hash: [u8], chan_id: u64, payment_proof: [u8], amount: u64, nonce: u64)
recipient claim for funds. It should provide proof that the claim operation is legal.

| Param     |  Description  |
|----------|------:|
| sender_pubkey_hash  | sender public key hash. used for identity proof in off-chain |
| chan_id  | payment channel id. each path sender -> recipient called a channel |
| payment_proof  | payment proof |
| amount  | claim amount |
| nonce  | payment channel nonce |

### 7. get_chan_info(sender_pubkey_hash: [u8], chan_id: u64) -> (per_face_value: u64, nonce: u64, max_value: u64)
get channel info by given chan_id
| Param     |  Description  |
|----------|------:|
| sender_pubkey_hash  | sender public key hash. used for identity proof in off-chain |
| chan_id  | payment channel id. each path sender -> recipient called a channel |
| per_face_value  |  face value per off-chain transaction |
| nonce  | channel nonce |
| max_value  | max value |

