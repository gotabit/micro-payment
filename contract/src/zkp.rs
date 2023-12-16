use ark_bn254::Bn254;
use ark_circom::{CircomBuilder, CircomConfig};
use ark_groth16::Groth16;
use ark_std::rand::thread_rng;
use color_eyre::Result;
use lazy_static::lazy_static;
use num_bigint::BigInt;

type _GrothBn = Groth16<Bn254>;

lazy_static! {
    pub static ref CIRCOM_CONFIG: CircomConfig<ark_ec::bn::Bn<ark_bn254::Config>> =
        CircomConfig::<Bn254>::new("./zkp_files/circuit.wasm", "./zkp_files/circuit.r1cs").unwrap();
}

#[warn(dead_code)]
pub fn _verify_proof(_pub_inputs: Vec<(impl ToString, impl Into<BigInt>)>) -> Result<()> {
    let mut _builder = CircomBuilder::new(CIRCOM_CONFIG.to_owned());

    let circom = _builder.setup();

    let mut rng = thread_rng();
    let _params = _GrothBn::generate_random_parameters_with_reduction(circom, &mut rng)?;

    let _circom: ark_circom::CircomCircuit<ark_ec::bn::Bn<ark_bn254::Config>> = _builder.build()?;

    Ok(())
}

#[test]
fn test_circom_config() {
    println!("{}", CIRCOM_CONFIG.r1cs.constraints.len());
}
