use std::{
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};

use crate::config::CONFIG;
use anyhow::{Context, Ok};
use pgp::{
    composed::{
        EncryptionCaps, KeyType, SecretKeyParamsBuilder, SignedPublicKey, SignedSecretKey,
        SubkeyParams, SubkeyParamsBuilder, SubkeyParamsBuilderError,
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

enum SubkeyType {
    Auth,
    Sign,
    Encrypt,
}

pub fn init() -> anyhow::Result<()> {
    let secret_key = keygen(
        KeyType::Ed25519,
        KeyType::Ed25519,
        KeyType::ECDH(ECCCurve::Curve25519),
        KeyType::Ed25519,
        "",
    )
    .context("failed during keygen")?;

    create_key(&secret_key, &GpgKeyType::Pub)?;
    create_key(&secret_key, &GpgKeyType::Priv)?;

    let server_key = exchange_keys(get_key_path(&GpgKeyType::Pub))
        .context("Couldn't exchange keys with server")?;

    save_key(server_key).context("Couldn't save key")
}

fn keygen(
    primary_key_type: KeyType,
    signing_key_type: KeyType,
    encryption_key_type: KeyType,
    auth_key_type: KeyType,
    uid: &str,
) -> anyhow::Result<SignedSecretKey> {
    let signkey = build_subkey(signing_key_type, SubkeyType::Sign)?;
    let encryptkey = build_subkey(encryption_key_type, SubkeyType::Encrypt)?;
    let authkey = build_subkey(auth_key_type, SubkeyType::Auth)?;

    let mut key_params_builder = SecretKeyParamsBuilder::default();
    key_params_builder
        .key_type(primary_key_type)
        .can_certify(true)
        .can_sign(false)
        .can_encrypt(EncryptionCaps::None)
        .primary_user_id(uid.into())
        .subkeys(vec![signkey, encryptkey, authkey]);

    let secret_key_params = key_params_builder
        .build()
        .context("Build secret_key_params")?;

    secret_key_params
        .generate(thread_rng())
        .context("Generate plain key")
}

fn build_subkey(
    key_type: KeyType,
    subkey_type: SubkeyType,
) -> Result<SubkeyParams, SubkeyParamsBuilderError> {
    let mut key = SubkeyParamsBuilder::default();

    key.key_type(key_type);

    match subkey_type {
        SubkeyType::Auth => {
            key.can_sign(false);
            key.can_encrypt(EncryptionCaps::None);
            key.can_authenticate(true);
        }
        SubkeyType::Sign => {
            key.can_sign(true);
            key.can_encrypt(EncryptionCaps::None);
            key.can_authenticate(false);
        }
        SubkeyType::Encrypt => {
            key.can_sign(false);
            key.can_encrypt(EncryptionCaps::All);
            key.can_authenticate(false);
        }
    }

    key.build()
}

fn exchange_keys(key: PathBuf) -> anyhow::Result<Response> {
    let server_route = format!("{}/keys-new/exchange", CONFIG.get().server.clone());

    let form = multipart::Form::new().file("publicKey", key)?;

    let client = Client::new();

    let res = client.post(server_route).multipart(form).send()?;

    res.error_for_status().map_err(anyhow::Error::from)
}

fn create_key(secret_key: &SignedSecretKey, key_type: &GpgKeyType) -> anyhow::Result<()> {
    fs::create_dir_all(CONFIG.get().keys.clone())?;

    let mut file = fs::File::create(get_key_path(key_type))?;

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
    let root = CONFIG.get().data_dir.clone();
    let keys = CONFIG.get().keys.clone();

    extract_zip(res, &keys)?;
    move_id_file(&root, &keys)?;
    move_key_file(&keys)?;

    fs::remove_file(keys.join("idfile")).map_err(anyhow::Error::from)
}

fn extract_zip(res: Response, keys: &PathBuf) -> anyhow::Result<()> {
    let cursor = Cursor::new(res.bytes()?);
    let mut zip = ZipArchive::new(cursor)?;

    ZipArchive::extract(&mut zip, keys).map_err(anyhow::Error::from)
}

fn move_id_file(root: &Path, keys: &Path) -> anyhow::Result<()> {
    let from = PathBuf::from(format!("{}/idfile", keys.display()));
    let to = PathBuf::from(format!("{}/uuid", root.display()));

    fs::copy(from, to).map_err(anyhow::Error::from).map(|_| ())
}

fn move_key_file(keys: &Path) -> anyhow::Result<()> {
    let from = PathBuf::from(format!("{}/public.asc", keys.display()));
    let to = PathBuf::from(format!("{}/server.pub", keys.display()));

    fs::rename(&from, &to).map_err(anyhow::Error::from)
}

fn get_key_path(key: &GpgKeyType) -> PathBuf {
    let keys_path = CONFIG.get().keys.clone();

    PathBuf::from(match key {
        GpgKeyType::Pub => format!("{}/key.pub", keys_path.display()),
        GpgKeyType::Priv => format!("{}/key", keys_path.display()),
    })
}
