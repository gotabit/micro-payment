pragma circom 2.0.0;

include "../../circuits/payment.circom";

component main {public [msg, sender_pubkey_hash]} = EdDSASignVerify();