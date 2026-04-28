use std::{
    error::Error,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use crate::config::CONFIG;
use pgp::{
    composed::{
        EncryptionCaps, KeyType, SecretKeyParamsBuilder, SignedPublicKey, SignedSecretKey,
        SubkeyParamsBuilder, SubkeyParamsBuilderError,
    },
    crypto::ecc_curve::ECCCurve,
};
use rand::thread_rng;
use reqwest::blocking::{multipart, Client, Response};
use zip::ZipArchive;

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

    let server_key = exchange_keys(pub_file_path.clone()).expect("Couldn't get server key");

    let server_key_path = CONFIG.get().keys.to_string_lossy().into_owned();

    let _saved_server_key = save_key(server_key, server_key_path).expect("Server key didn't save");
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

fn exchange_keys(key: impl AsRef<str>) -> anyhow::Result<Response> {
    let server = CONFIG.get().server.clone();

    let server = "http://localhost:5678";

    let form = multipart::Form::new().file("publicKey", key.as_ref())?;

    let client = Client::new();

    let res = client
        .post(format!("{}/keys/exchange", &server))
        .multipart(form)
        .send()?;

    Ok(res)
}

fn save_key(mut res: Response, keys_path: impl AsRef<str>) -> anyhow::Result<()> {
    // /var/lib/relago
    let data_dir = CONFIG
        .get()
        .data_dir
        .clone()
        .into_os_string()
        .into_string()
        .unwrap();

    let keys_path = keys_path.as_ref();

    // Extraction

    let zip_file_path = PathBuf::from(format!("{}/res.zip", keys_path));

    let mut created_file = File::create(&zip_file_path)?;

    res.copy_to(&mut created_file)?;

    let opened_file = File::open(&zip_file_path)?;

    let mut zip = ZipArchive::new(&opened_file)?;

    let mut _extracted = ZipArchive::extract(&mut zip, &keys_path);

    // Moving id file
    let from_id_path = PathBuf::from(format!("{}/idfile", keys_path));

    let to_id_path = PathBuf::from(format!("{}/user", &data_dir));

    let _is_id_copied = fs::copy(from_id_path, to_id_path);

    // Moving key file

    let from_key_path = PathBuf::from(format!("{}/public.asc", keys_path));

    let to_key_path = PathBuf::from(format!("{}/server.pub", keys_path));

    let _is_key_renamed = fs::rename(&from_key_path, &to_key_path);

    // Deleting garbage
    let _is_deleted = fs::remove_file(&zip_file_path);

    Ok(())
}
