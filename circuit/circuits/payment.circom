pragma circom 2.0.0;

include "../circomlib/circuits/poseidon.circom";
include "../circomlib/circuits/eddsaposeidon.circom";

template EdDSASignVerify() {
    // pubkey field
    signal input Ax;
    signal input Ay;
    // signature field
    signal input S;
    signal input R8x;
    signal input R8y;
    // sign msg
    signal input msg;

    signal input sender_pubkey_hash;

    /// constraint: sender_pubkey_hash === Poseidon(sender_pubkey)
    component poseidon_hash = Poseidon(2);

    poseidon_hash.inputs[0] <== Ax;
    poseidon_hash.inputs[1] <== Ay;
    poseidon_hash.out === sender_pubkey_hash;

    // verify eddsa signature
    component eddsa_verify = EdDSAPoseidonVerifier();

    eddsa_verify.enabled <== 1;
    eddsa_verify.Ax <== Ax;
    eddsa_verify.Ay <== Ay;
    eddsa_verify.S <== S;
    eddsa_verify.R8x <== R8x;
    eddsa_verify.R8y <== R8y;
    eddsa_verify.M <== msg;
}

component main {public [msg, sender_pubkey_hash]} = EdDSASignVerify();