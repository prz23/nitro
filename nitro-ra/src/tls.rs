use ring::{
    rand,
    signature::{self, KeyPair},
};
use yasna::models::ObjectIdentifier;
use chrono::Duration;
use chrono::TimeZone;
use chrono::Utc as TzUtc;
use std::time::*;
use bit_vec::BitVec;
use num_bigint::BigUint;

pub const CERTEXPIRYDAYS: i64 = 90i64;
const ISSUER : &str = "MesaTEE";
const SUBJECT : &str = "MesaTEE";

pub fn ring_key_gen_pcks_8() -> (signature::EcdsaKeyPair,Vec<u8>){
    let rng = rand::SystemRandom::new();

    let key_pair = signature::EcdsaKeyPair::generate_pkcs8(&signature::ECDSA_P256_SHA256_ASN1_SIGNING, &rng).unwrap();
    let res = signature::EcdsaKeyPair::from_pkcs8(&signature::ECDSA_P256_SHA256_ASN1_SIGNING,key_pair.as_ref()).unwrap();
    println!("========key============");
    (res,key_pair.as_ref().to_vec())
}

pub fn gen_ecc_cert(payload: String,
                    prv_k: signature::EcdsaKeyPair , key_pair: Vec<u8>) -> Result<Vec<u8>, String> {
    // Generate public key bytes since both DER will use it
    let mut pub_key_bytes: Vec<u8> = Vec::with_capacity(0);
    pub_key_bytes.extend_from_slice(&key_pair);
    println!("==pub_key_bytes=={:?}",pub_key_bytes);
    // Generate Certificate DER
    let cert_der = yasna::construct_der(|writer| {
        writer.write_sequence(|writer| {
            writer.next().write_sequence(|writer| {
                // Certificate Version
                writer.next().write_tagged(yasna::Tag::context(0), |writer| {
                    writer.write_i8(2);
                });
                // Certificate Serial Number (unused but required)
                writer.next().write_u8(1);
                // Signature Algorithm: ecdsa-with-SHA256
                writer.next().write_sequence(|writer| {
                    writer.next().write_oid(&ObjectIdentifier::from_slice(&[1,2,840,10045,4,3,2]));
                });
                // Issuer: CN=MesaTEE (unused but required)
                writer.next().write_sequence(|writer| {
                    writer.next().write_set(|writer| {
                        writer.next().write_sequence(|writer| {
                            writer.next().write_oid(&ObjectIdentifier::from_slice(&[2,5,4,3]));
                            writer.next().write_utf8_string(&ISSUER);
                        });
                    });
                });
                // Validity: Issuing/Expiring Time (unused but required)
                let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
                let issue_ts = TzUtc.timestamp(now.as_secs() as i64, 0);
                let expire = now + Duration::days(CERTEXPIRYDAYS).to_std().unwrap();
                let expire_ts = TzUtc.timestamp(expire.as_secs() as i64, 0);
                writer.next().write_sequence(|writer| {
                    writer.next().write_utctime(&yasna::models::UTCTime::from_datetime(&issue_ts));
                    writer.next().write_utctime(&yasna::models::UTCTime::from_datetime(&expire_ts));
                });
                // Subject: CN=MesaTEE (unused but required)
                writer.next().write_sequence(|writer| {
                    writer.next().write_set(|writer| {
                        writer.next().write_sequence(|writer| {
                            writer.next().write_oid(&ObjectIdentifier::from_slice(&[2,5,4,3]));
                            writer.next().write_utf8_string(&SUBJECT);
                        });
                    });
                });
                writer.next().write_sequence(|writer| {
                    // Public Key Algorithm
                    writer.next().write_sequence(|writer| {
                        // id-ecPublicKey
                        writer.next().write_oid(&ObjectIdentifier::from_slice(&[1,2,840,10045,2,1]));
                        // prime256v1
                        writer.next().write_oid(&ObjectIdentifier::from_slice(&[1,2,840,10045,3,1,7]));
                    });
                    // Public Key
                    writer.next().write_bitvec(&BitVec::from_bytes(&pub_key_bytes));
                });
                // Certificate V3 Extension
                writer.next().write_tagged(yasna::Tag::context(3), |writer| {
                    writer.write_sequence(|writer| {
                        writer.next().write_sequence(|writer| {
                            writer.next().write_oid(&ObjectIdentifier::from_slice(&[2,16,840,1,113730,1,13]));
                            writer.next().write_bytes(&payload.into_bytes());
                        });
                    });
                });
            });
            // Signature Algorithm: ecdsa-with-SHA256
            writer.next().write_sequence(|writer| {
                writer.next().write_oid(&ObjectIdentifier::from_slice(&[1,2,840,10045,4,3,2]));
            });
            // Signature
            let sig = {
                let tbs = &writer.buf[4..];
                // ecc_handle.ecdsa_sign_slice(tbs, &prv_k).unwrap()
                let rng = rand::SystemRandom::new();
                prv_k.sign(&rng,&tbs.to_vec()).unwrap().as_ref().to_vec()
            };
            let sig_der = yasna::construct_der(|writer| {
                writer.write_sequence(|writer| {
                    //let mut sig_x = sig.x.clone();
                    let mut sig_x = sig[..32].to_vec();
                    sig_x.reverse();
                    //let mut sig_y = sig.y.clone();
                    let mut sig_y = sig[32..].to_vec();
                    sig_y.reverse();
                    writer.next().write_biguint(&BigUint::from_bytes_be(&sig_x));
                    writer.next().write_biguint(&BigUint::from_bytes_be(&sig_y));
                });
            });
            writer.next().write_bitvec(&BitVec::from_bytes(&sig_der));
        });
    });

    Ok(cert_der)
}