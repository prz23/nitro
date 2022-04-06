mod tls;
pub use tls::{ring_key_gen_pcks_8,gen_ecc_cert};

use nsm_io::{Request, Response, AttestationDoc};
use serde_bytes::ByteBuf;

use serde::{Deserialize, Serialize};
use serde_cbor::error::Error as CborError;
use serde_cbor::{from_slice, to_vec};

use webpki;
use base64;

use std::io::BufReader;
use std::time::SystemTime;

// use nitro_enclave_attestation_document;

pub fn get_remote_attestation() -> Response{
    let nsm_fd = nsm_driver::nsm_init();

    let public_key = ByteBuf::from("my super secret key");
    let hello = ByteBuf::from("hello, world!");

    let request = Request::Attestation {
        public_key: Some(public_key),
        user_data: Some(hello),
        nonce: None,
    };

    let response = nsm_driver::nsm_process_request(nsm_fd, request);
    println!("{:?}", response);

    nsm_driver::nsm_exit(nsm_fd);

    response
}

pub fn resolve_the_response(response:Response) -> Result<Vec<u8>,String> {
    let document = match response {
        Response::Attestation { document } => { document },
        _ => { return Err("the response is not Attestation".to_string()); },
    };

    let data: AttestationDoc = serde_cbor::from_slice(&document).unwrap();
    let certificate = data.certificate.into_vec();

    Ok(certificate)
}

pub fn verify_the_certificate(){
}

pub const CA : &[u8] = include_bytes!("root.pem");
type SignatureAlgorithms = &'static [&'static webpki::SignatureAlgorithm];
static SUPPORTED_SIG_ALGS: SignatureAlgorithms = &[
    &webpki::ECDSA_P256_SHA256,
    &webpki::ECDSA_P256_SHA384,
    &webpki::ECDSA_P384_SHA256,
    &webpki::ECDSA_P384_SHA384,
    &webpki::RSA_PSS_2048_8192_SHA256_LEGACY_KEY,
    &webpki::RSA_PSS_2048_8192_SHA384_LEGACY_KEY,
    &webpki::RSA_PSS_2048_8192_SHA512_LEGACY_KEY,
    &webpki::RSA_PKCS1_2048_8192_SHA256,
    &webpki::RSA_PKCS1_2048_8192_SHA384,
    &webpki::RSA_PKCS1_2048_8192_SHA512,
    &webpki::RSA_PKCS1_3072_8192_SHA384,
];

pub fn verify_cert(sig_raw:Vec<u8>, sig_cert_raw:Vec<u8>, attn_report_raw:Vec<u8>){

    let sig = base64::decode(&sig_raw).unwrap();
    let sig_cert_dec = base64::decode_config(&sig_cert_raw, base64::MIME).unwrap();
    let sig_cert = webpki::EndEntityCert::from(&sig_cert_dec).expect("Bad DER");

    let mut ias_ca_stripped = CA.to_vec();
    ias_ca_stripped.retain(|&x| x != 0x0d && x != 0x0a);
    let head_len = "-----BEGIN CERTIFICATE-----".len();
    let tail_len = "-----END CERTIFICATE-----".len();
    let full_len = ias_ca_stripped.len();
    let ias_ca_core : &[u8] = &ias_ca_stripped[head_len..full_len - tail_len];
    let ias_cert_dec = base64::decode_config(ias_ca_core, base64::MIME).unwrap();

    let mut ca_reader = BufReader::new(&CA[..]);

    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_pem_file(&mut ca_reader).expect("Failed to add CA");

    let trust_anchors: Vec<webpki::TrustAnchor> = root_store
        .roots
        .iter()
        .map(|cert| cert.to_trust_anchor())
        .collect();

    let mut chain:Vec<&[u8]> = Vec::new();
    chain.push(&ias_cert_dec);

    let now_func = webpki::Time::try_from(SystemTime::now());

    match sig_cert.verify_is_valid_tls_server_cert(
        SUPPORTED_SIG_ALGS,
        &webpki::TLSServerTrustAnchors(&trust_anchors),
        &chain,
        now_func.unwrap()) {
        Ok(_) => println!("Cert is good"),
        Err(e) => println!("Cert verification error {:?}", e),
    }

    // Verify the signature against the signing cert
    match sig_cert.verify_signature(
        &webpki::RSA_PKCS1_2048_8192_SHA256,
        &attn_report_raw,
        &sig) {
        Ok(_) => println!("Signature good"),
        Err(e) => {
            println!("Signature verification error {:?}", e);
            panic!();
        },
    }

}