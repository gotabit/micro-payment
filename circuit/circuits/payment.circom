pragma circom 2.0.0;

include "../circomlib/circuits/poseidon.circom";
include "../circom-ecdsa/circuits/ecdsa.circom";

template PaymentVerify(n, k) {
    // private input
    signal input sender_pubkey[2][k];
    // signature (r, s)
    signal input r[k];
    signal input s[k];
    /// public input
    signal input msghash[k];
    signal input sender_pubkey_hash;
    // public input

    /// constraint: sender_pubkey_hash === Poseidon(sender_pubkey)
    component poseidon_hash = Poseidon(2 * k);

    for (var i = 0; i < k; i++) {
      poseidon_hash.inputs[i] <== sender_pubkey[0][i];
      poseidon_hash.inputs[k + i] <== sender_pubkey[1][i];
    }

    // log("poseidon_hash: ", poseidon_hash.out);
    poseidon_hash.out === sender_pubkey_hash;

    // constraint: verify_sign((r, s), msghash, sender_pubkey) === true
    component sign_verify = ECDSAVerifyNoPubkeyCheck(n, k);

    for (var i = 0; i < k; i++) {
      sign_verify.r[i] <== r[i];
      sign_verify.s[i] <== s[i];
      sign_verify.msghash[i] <== msghash[i];
      sign_verify.pubkey[0][i] <== sender_pubkey[0][i];
      sign_verify.pubkey[1][i] <== sender_pubkey[1][i];
    }

    sign_verify.result === 1;
}


// component main {public [msghash, sender_pubkey_hash]} = PaymentVerify(64, 4);