use nsm_io::{Request, Response,AttestationDoc};
use serde_bytes::ByteBuf;

use serde::{Deserialize, Serialize};
use serde_cbor::error::Error as CborError;
use serde_cbor::{from_slice, to_vec};

use webpki;

use std::io::BufReader;



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
    let sig_cert = webpki::EndEntityCert::from(&sig_cert_dec).expect("Bad DER");

}

pub const CA : &[u8] = include_bytes!("./AttestationReportSigningCACert.pem");

pub fn load_ca(path:String){
    let mut ca_reader = BufReader::new(&IAS_REPORT_CA[..]);

    let mut root_store = rustls::RootCertStore::empty();
    root_store.add_pem_file(&mut ca_reader).expect("Failed to add CA");

    let trust_anchors: Vec<webpki::TrustAnchor> = root_store
        .roots
        .iter()
        .map(|cert| cert.to_trust_anchor())
        .collect();

    let mut chain:Vec<&[u8]> = Vec::new();
    chain.push(&ias_cert_dec);
}