use crate::HostData;
use boa_engine::object::builtins::{JsPromise, JsUint8Array};
use boa_engine::{js_error, js_string, Context, JsError, JsResult, JsString, JsValue, Module};
use boa_interop::{ContextData, IntoJsFunctionCopied, IntoJsModule};
use ed25519::pkcs8::DecodePublicKey;
use ed25519::signature::Verifier;
use firmware_ui::application::panels::alert::alert;
use std::os::unix::process::CommandExt;
use std::process::Command;

// Using `.pub` as extension to avoid security scanner thinking this is a
// private key.
const PUBLIC_KEY_PEM: &str = include_str!("../../../assets/1fpga-release.pub");

fn verify_inner_(path: String, signature: JsUint8Array, context: &mut Context) -> JsResult<bool> {
    let buffer = std::fs::read(path).map_err(JsError::from_rust)?;
    let signature = signature.iter(context).collect::<Vec<u8>>();
    let signature =
        ed25519_dalek::Signature::try_from(signature.as_slice()).map_err(JsError::from_rust)?;

    let public_key = ed25519_dalek::VerifyingKey::from_public_key_pem(PUBLIC_KEY_PEM)
        .map_err(JsError::from_rust)?;

    Ok(public_key
        .verify(buffer.as_ref(), &signature)
        .map_or(false, |_| true))
}

fn verify_signature_(path: String, signature: JsUint8Array, context: &mut Context) -> JsPromise {
    match verify_inner_(path, signature, context) {
        Ok(result) => JsPromise::resolve(JsValue::from(result), context),
        Err(err) => JsPromise::reject(err, context),
    }
}

fn upgrade_(
    ContextData(host_data): ContextData<HostData>,
    name: String,
    path: String,
    signature: Option<JsUint8Array>,
    context: &mut Context,
) -> JsPromise {
    // Only support 1fpga upgrade right now.
    if name != "1fpga" {
        return JsPromise::reject(js_error!("Unsupported upgrade"), context);
    }

    if let Some(sig) = signature {
        match verify_inner_(path.clone(), sig, context) {
            Ok(true) => (),
            Ok(false) => {
                return JsPromise::reject(js_error!("Invalid signature"), context);
            }
            Err(err) => {
                return JsPromise::reject(err, context);
            }
        }
    } else {
        // No signature provided, double check with the user.
        let choice = alert(
            host_data.app_mut(),
            "No signature provided",
            "\
                1FPGA cannot verify the upgrade. \
                Make sure you trust the source.\n\n\
                Do you want to continue with the upgrade?
            ",
            &["Cancel the upgrade", "Trust the source and continue"],
        );

        // We do not error on cancel.
        if choice != Some(1) {
            return JsPromise::resolve(JsValue::undefined(), context);
        }
    }

    let current_path = match std::env::current_exe() {
        Ok(c) => c,
        Err(e) => return JsPromise::reject(JsError::from_rust(e), context),
    };
    let backup = current_path.with_extension("bak");
    if backup.exists() {
        return JsPromise::reject(
            js_error!("Backup already exists, please remove it first"),
            context,
        );
    }

    match std::fs::rename(&current_path, format!("{}.bak", current_path.display())) {
        Ok(_) => (),
        Err(e) => return JsPromise::reject(JsError::from_rust(e), context),
    }
    match std::fs::copy(&path, &current_path) {
        Ok(_) => (),
        Err(e) => {
            // Try to restore the backup.
            match std::fs::rename(format!("{}.bak", current_path.display()), &current_path) {
                Ok(_) => (),
                Err(e2) => {
                    return JsPromise::reject(JsError::from_rust(e2), context);
                }
            }
            return JsPromise::reject(JsError::from_rust(e), context);
        }
    }

    let _ = std::fs::remove_file(format!("{}.bak", current_path.display()));
    let _ = std::fs::remove_file(path);

    // Restart the binary. This will never return unless there's an error.
    let err = Command::new(&current_path)
        .args(std::env::args().skip(1))
        .exec();

    JsPromise::reject(JsError::from_rust(err), context)
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("upgrade"),
        [
            (
                js_string!("upgrade"),
                upgrade_.into_js_function_copied(context),
            ),
            (
                js_string!("verifySignature"),
                verify_signature_.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}
