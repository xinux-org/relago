use std::{
    fs::{self, File},
    path::PathBuf,
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

#[derive(Clone)]
enum GpgKeyType {
    Pub,
    Priv,
}

pub fn init() -> anyhow::Result<()> {
    let secret_key = keygen(
        KeyType::Ed25519,
        KeyType::Ed25519,
        KeyType::ECDH(ECCCurve::Curve25519),
        KeyType::Ed25519,
        "",
    )
    .expect("failed during keygen");

    let _is_pub_key_created = create_key(&secret_key, GpgKeyType::Pub);
    let _is_priv_key_created = create_key(&secret_key, GpgKeyType::Priv);

    let server_key =
        exchange_keys(get_key_path(GpgKeyType::Pub)).expect("Couldn't exchange keys with server");

    let _is_server_key_saved = save_key(server_key)?;

    Ok(())
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

fn exchange_keys(key: PathBuf) -> anyhow::Result<Response> {
    let server_route = format!("{}/keys/exchange", CONFIG.get().server.clone());

    let form = multipart::Form::new().file("publicKey", key)?;

    let client = Client::new();

    let res = client.post(server_route).multipart(form).send()?;

    Ok(res)
}

fn create_key(secret_key: &SignedSecretKey, key_type: GpgKeyType) -> anyhow::Result<()> {
    let _is_keys_dir_created = fs::create_dir_all(CONFIG.get().keys.clone())?;

    let mut file = fs::File::create(get_key_path(key_type.clone()))?;

    match key_type {
        GpgKeyType::Priv => {
            secret_key.to_armored_writer(&mut file, None.into())?;
        }
        GpgKeyType::Pub => {
            let public_key = SignedPublicKey::from(secret_key.clone());
            public_key.to_armored_writer(&mut file, None.into())?;
        }
    }

    Ok(())
}

fn save_key(res: Response) -> anyhow::Result<()> {
    // /var/lib/relago
    let root = CONFIG.get().data_dir.clone();
    let keys = CONFIG.get().keys.clone();
    let zip = PathBuf::from(format!("{}/res.zip", &keys.display()));

    // Extraction zip
    let _is_extracted = extract_zip(res, &zip, &keys);

    // Moving id file
    let _is_id_file_moved = move_id_file(&root, &keys);

    // Moving key file
    let _is_key_file_moved = move_key_file(&keys);

    // Deleting garbage
    let _is_deleted = fs::remove_file(&zip);

    Ok(())
}

fn extract_zip(mut res: Response, zip: &PathBuf, keys: &PathBuf) -> anyhow::Result<()> {
    let mut created_file = File::create(zip)?;

    res.copy_to(&mut created_file)?;

    let opened_file = File::open(zip)?;

    let mut zip = ZipArchive::new(&opened_file)?;

    let mut _extracted = ZipArchive::extract(&mut zip, &keys);

    Ok(())
}

fn move_id_file(root: &PathBuf, keys: &PathBuf) -> anyhow::Result<()> {
    let from = PathBuf::from(format!("{}/idfile", keys.display()));

    let to = PathBuf::from(format!("{}/user", root.display()));

    let _is_id_copied = fs::copy(from, to);

    Ok(())
}

fn move_key_file(keys: &PathBuf) -> anyhow::Result<()> {
    let from = PathBuf::from(format!("{}/public.asc", keys.display()));

    let to = PathBuf::from(format!("{}/server.pub", keys.display()));

    let _is_key_moved = fs::rename(&from, &to);

    Ok(())
}

fn get_key_path(key: GpgKeyType) -> PathBuf {
    let keys_path = CONFIG.get().keys.clone();

    PathBuf::from(match key {
        GpgKeyType::Pub => format!("{}/key.pub", keys_path.display()),
        GpgKeyType::Priv => format!("{}/key", keys_path.display()),
    })
}
