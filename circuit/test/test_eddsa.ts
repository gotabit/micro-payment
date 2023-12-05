import path = require("path");

import { expect, assert } from 'chai';
const circom_tester = require('circom_tester');
const wasm_tester = circom_tester.wasm;

const F1Field = require("ffjavascript").F1Field;
const Scalar = require("ffjavascript").Scalar;
exports.p = Scalar.fromString("21888242871839275222246405745257275088548364400416034343698204186575808495617");
const Fr = new F1Field(exports.p);
const buildEddsa = require("circomlibjs").buildEddsa;
const buildBabyjub = require("circomlibjs").buildBabyjub;
const buildPoseidon = require("circomlibjs").buildPoseidon;

describe("EdDSA Poseidon test", function () {
    let circuit: any;
    let eddsa: any;
    let babyJub: any;
    let F: any;
    let poseidon: any;

    this.timeout(100000);

    before(async () => {
        eddsa = await buildEddsa();
        babyJub = await buildBabyjub();
        F = babyJub.F;
        poseidon = await buildPoseidon();
        circuit = await wasm_tester(path.join(__dirname, "circuits", "test_eddsa.circom"));
    });

    it("Sign a single number", async () => {
        const msg = F.e(123);

        const prvKey = Buffer.from("0001020304050607080900010203040506070809000102030405060708090001", "hex");

        const pubKey = eddsa.prv2pub(prvKey);

        const signature = eddsa.signPoseidon(prvKey, msg);

        const p_hash = poseidon(pubKey);

        assert(eddsa.verifyPoseidon(msg, signature, pubKey));

        const input = {
            Ax: F.toObject(pubKey[0]),
            Ay: F.toObject(pubKey[1]),
            R8x: F.toObject(signature.R8[0]),
            R8y: F.toObject(signature.R8[1]),
            S: signature.S,
            msg: F.toObject(msg),
            sender_pubkey_hash: F.toObject(p_hash),
        };

        const w = await circuit.calculateWitness(input, true);

        await circuit.checkConstraints(w);
    });
});