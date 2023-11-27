pragma circom 2.0.0;

include "../../circuits/payment.circom";

component main {public [msghash, sender_pubkey_hash]} = PaymentVerify(64, 4);