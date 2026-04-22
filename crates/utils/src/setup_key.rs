use std::{error::Error, fs::File, io::Read};

use crate::config::CONFIG;
use pgp::{
    composed::{
        EncryptionCaps, KeyType, SecretKeyParamsBuilder, SignedPublicKey, SignedSecretKey,
        SubkeyParamsBuilder, SubkeyParamsBuilderError,
    },
    crypto::ecc_curve::ECCCurve,
};
use rand::thread_rng;
use reqwest::blocking::multipart;

pub fn init() {
    let write_path = CONFIG.get().keys.to_str().unwrap();

    let secret_key = keygen(
        KeyType::Ed25519,
        KeyType::Ed25519,
        KeyType::ECDH(ECCCurve::Curve25519),
        KeyType::Ed25519,
        "",
    )
    .expect("failed during keygen");

    let mut priv_file = std::fs::File::create(format!("{}/user.priv", write_path))
        .expect("failed to create 'example-key.priv'");
    secret_key
        .to_armored_writer(&mut priv_file, None.into())
        .expect("failed to write to 'example-key.priv'");

    let public_key = SignedPublicKey::from(secret_key.clone());

    let pub_file_path = format!("{}/user.pub", write_path);

    let mut pub_file =
        std::fs::File::create(&pub_file_path).expect("failed to create 'example-key.pub'");
    public_key
        .to_armored_writer(&mut pub_file, None.into())
        .expect("failed to write to 'example-key.pub'");

    send_key(pub_file_path.clone());
}

fn keygen(
    primary_key_type: KeyType,
    signing_key_type: KeyType,
    encryption_key_type: KeyType,
    auth_key_type: KeyType,
    uid: &str,
) -> Result<SignedSecretKey, SubkeyParamsBuilderError> {
    let mut signkey = SubkeyParamsBuilder::default();
    signkey
        .key_type(signing_key_type)
        .can_sign(true)
        .can_encrypt(EncryptionCaps::None)
        .can_authenticate(false);
    let mut encryptkey = SubkeyParamsBuilder::default();
    encryptkey
        .key_type(encryption_key_type)
        .can_sign(false)
        .can_encrypt(EncryptionCaps::All)
        .can_authenticate(false);
    let mut authkey = SubkeyParamsBuilder::default();
    authkey
        .key_type(auth_key_type)
        .can_sign(false)
        .can_encrypt(EncryptionCaps::None)
        .can_authenticate(true);

    let mut key_params = SecretKeyParamsBuilder::default();
    key_params
        .key_type(primary_key_type)
        .can_certify(true)
        .can_sign(false)
        .can_encrypt(EncryptionCaps::None)
        .primary_user_id(uid.into())
        .subkeys(vec![
            signkey.build()?,
            encryptkey.build()?,
            authkey.build()?,
        ]);

    let secret_key_params = key_params.build().expect("Build secret_key_params");

    let signed = secret_key_params
        .generate(thread_rng())
        .expect("Generate plain key");

    Ok(signed)
}

async fn send_key(key: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let server = CONFIG.get().server.clone();

    let server = "http://localhost:4242";

    let form = multipart::Form::new().file("publicKey", key)?;
    let mut res = reqwest::blocking::Client::new()
        .post(format!("{}/keys/exchange", &server))
        .multipart(form)
        .send()?;

    println!();

    Ok(())
}
