use crate::args::Args;
use bincode::serde::{decode_from_slice, encode_to_vec};
use chrono::{Local, NaiveDate};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use hex::{decode, encode};
use rand_chacha::rand_core::OsRng;
use rust_to_ts_types::TypescriptSerializable;
use serde::{Deserialize, Serialize};

pub fn dev_mode() -> bool {
    let profile = env!("PROFILE");

    profile != "release"
}

pub fn today() -> NaiveDate {
    let today: NaiveDate = Local::now().date_naive();
    today
}

const CONFIG: bincode::config::Configuration<
    bincode::config::LittleEndian,
    bincode::config::Fixint,
> = bincode::config::standard()
    .with_little_endian()
    .with_fixed_int_encoding();

#[derive(Serialize, Deserialize, Clone, TypescriptSerializable, Debug)]
pub struct ProductKey {
    start_date: NaiveDate,
    end_date: NaiveDate,
    company_name: String,
}

#[derive(Serialize, Deserialize, Clone, TypescriptSerializable)]
struct ProductKeyContainer {
    key: Vec<u8>,
    signature: Vec<u8>,
}

type MasterKeySecretPart = String;
type MasterKeyPublicPart = String;

fn generate_product_key_system() -> (MasterKeySecretPart, MasterKeyPublicPart) {
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);

    let secret_key = encode(signing_key.as_bytes());
    let public_key = encode(signing_key.verifying_key().as_bytes());
    return (secret_key, public_key);
}

pub fn initialize_product_key_system(args: &Args) {
    match public_key_part() {
        Err(e) => {
            // initial generation of secret/public key
            error!("Failed to reat public verifying key, du to: {}", e);
            debug!("Generating new key pair");

            let (master_key_secret, master_key_public): (MasterKeySecretPart, MasterKeyPublicPart) =
                generate_product_key_system();

            info!("New master key generated. Secret part for SECRET storage: {}, public part for the code: {}", master_key_secret, master_key_public);
        }
        Ok(_) => {
            // check if provided key is valid
            match &args.product_key {
                Some(product_key) => match product_key_valid(Some(product_key)) {
                    Ok(verified_key) => {
                        info!(
                            "Program was provided a key that could be verified: {:?}",
                            verified_key
                        );
                    }
                    Err(e) => {
                        error!(
                            "Product key was provided for checking but verification failed: {}",
                            e
                        )
                    }
                },
                None => {
                    info!("No product key provided to check")
                }
            }

            // potentially generate a new key (if data for that is supplied)
            if let Some(start_date) = &args.generate_key_start_date {
                if let Some(end_date) = &args.generate_key_end_date {
                    if let Some(company_name) = &args.generate_key_company_name {
                        if let Some(master_key) = &args.master_secret {
                            match generate_new_product_key(
                                master_key,
                                company_name,
                                start_date,
                                end_date,
                            ) {
                                Ok(new_product_key) => {
                                    info!("Generated a new product key: {}", new_product_key);
                                }
                                Err(e) => {
                                    error!("Failed to generate a new product key: {}", e);
                                }
                            }
                            return;
                        }
                    }
                }
            }
            warn!("No new product key was generated, because inputs were missing");
        }
    }
}

fn public_key_part() -> Result<VerifyingKey, String> {
    // return Err("No public key hardcoded".into());

    let hardcoded_key = "a83555080f6dd185d2fbb0c2594f3cd224af08be4ae856880e31b411bbed2ab1";
    match decode(hardcoded_key) {
        Ok(key_data) => {
            let arr_ref: &[u8; 32] = match key_data.as_slice().try_into() {
                Ok(a) => a,
                Err(e) => {
                    return Err(format!(
                        "Could not convert hardcoded bytes data: {}",
                        e.to_string()
                    ))
                }
            };

            match VerifyingKey::from_bytes(arr_ref) {
                Ok(key) => return Ok(key),
                Err(e) => {
                    return Err(format!(
                        "Could not build key from hardcoded data: {}",
                        e.to_string()
                    ))
                }
            }
        }
        Err(e) => return Err(format!("Could not decode hardcoded key: {}", e.to_string())),
    }
}

fn generate_new_product_key(
    master_key: &MasterKeySecretPart,
    company_name: &str,
    start_date: &NaiveDate,
    end_date: &NaiveDate,
) -> Result<String, String> {
    let key_data = match decode(master_key) {
        Ok(key_data) => key_data,
        Err(e) => return Err(format!("Could not decode master secret: {}", e.to_string())),
    };
    let key_bytes: &[u8; 32] = match key_data.as_slice().try_into() {
        Ok(a) => a,
        Err(e) => {
            return Err(format!(
                "Could not convert master secret bytes data: {}",
                e.to_string()
            ))
        }
    };
    let signing_key = SigningKey::from_bytes(key_bytes);

    let product_key_struct = ProductKey {
        company_name: String::from(company_name),
        start_date: start_date.clone(),
        end_date: end_date.clone(),
    };

    let product_key_vec = match encode_to_vec(product_key_struct, CONFIG) {
        Ok(data) => data,
        Err(e) => {
            return Err(format!(
                "Could not encode a newly generated product key: {}",
                e.to_string()
            ))
        }
    };

    let signature = signing_key.sign(&product_key_vec);
    let container = ProductKeyContainer {
        key: product_key_vec,
        signature: signature.to_vec(),
    };

    let container_vec = match encode_to_vec(container, CONFIG) {
        Ok(data) => data,
        Err(e) => {
            return Err(format!(
                "Could not encode a newly generated product key (container stage): {}",
                e.to_string()
            ))
        }
    };

    return Ok(encode(container_vec));
}

pub fn product_key_valid(key: Option<&String>) -> Result<ProductKey, String> {
    let key_container = match key {
        Some(key) => match decode(key) {
            Ok(bytes) => match decode_from_slice::<ProductKeyContainer, _>(&bytes, CONFIG) {
                Ok((key, _)) => Some(key),
                Err(e) => {
                    return Err(format!(
                        "Product key could not be decoded (container stage): {}",
                        e.to_string()
                    ))
                }
            },
            Err(e) => {
                return Err(format!(
                    "Product key could not be decoded (hex stage): {}",
                    e.to_string()
                ))
            }
        },
        None => None,
    };

    if let Some(key_container) = key_container {
        match decode_from_slice::<ProductKey, _>(&key_container.key, CONFIG) {
            Ok((key, _)) => match public_key_part() {
                Ok(verifying_key) => {
                    let arr_ref: &[u8; 64] = match key_container.signature.as_slice().try_into() {
                        Ok(a) => a,
                        Err(e) => {
                            return Err(format!(
                                "Could not convert signature bytes: {}",
                                e.to_string()
                            ))
                        }
                    };

                    match verifying_key
                        .verify_strict(&key_container.key, &Signature::from_bytes(arr_ref))
                    {
                        Ok(()) => {
                            // product key is a validly signed key.
                            if key.start_date > today() {
                                return Err(
                                    "Key is not valid, as start date is in the future".into()
                                );
                            }
                            if key.end_date < today() {
                                return Err("Key is not valid, as end date is in the past".into());
                            }

                            return Ok(key);
                        }
                        Err(e) => {
                            return Err(format!(
                                "Product key signature verification failed: {}",
                                e.to_string()
                            ))
                        }
                    }
                }
                Err(e) => {
                    return Err(format!(
                        "Could not get hardcoded public verifier key: {}",
                        e.to_string()
                    ))
                }
            },
            Err(e) => {
                return Err(format!(
                    "Product key could not be decoded (content stage): {}",
                    e.to_string()
                ))
            }
        }
    }

    if dev_mode() {
        return Ok({
            ProductKey {
                start_date: NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(2200, 1, 1).unwrap(),
                company_name: "Development".into(),
            }
        });
    }

    return Err("No product key provided".into());
}
