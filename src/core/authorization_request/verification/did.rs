use std::str::FromStr;

use crate::core::{
    authorization_request::AuthorizationRequestObject,
    metadata::{parameters::wallet::RequestObjectSigningAlgValuesSupported, WalletMetadata},
    object::ParsingErrorContext,
};
use anyhow::{bail, Context, Result};
use base64::prelude::*;
use serde_json::{Map, Value as Json};
use ssi::{
    dids::{DIDBuf, DIDResolver, VerificationMethodDIDResolver},
    jwk::JWKResolver,
    verification_methods::{
        GenericVerificationMethod, InvalidVerificationMethod, MaybeJwkVerificationMethod,
        VerificationMethodSet,
    },
    JWK,
};

/// Default implementation of request validation for `client_id_scheme` `did`.
pub async fn verify_with_resolver(
    wallet_metadata: &WalletMetadata,
    request_object: &AuthorizationRequestObject,
    request_jwt: String,
    trusted_dids: Option<&[String]>,
    // resolver: &VerificationMethodDIDResolver<impl DIDResolver, M>,
    resolver: impl DIDResolver,
) -> Result<()>
// where
//     M: MaybeJwkVerificationMethod
//         + VerificationMethodSet
//         + TryFrom<GenericVerificationMethod, Error = InvalidVerificationMethod>,
{
    let (headers_b64, _, _) = ssi::claims::jws::split_jws(&request_jwt)?;

    let headers_json_bytes = BASE64_URL_SAFE_NO_PAD
        .decode(headers_b64)
        .context("jwt headers were not valid base64url")?;

    let mut headers = serde_json::from_slice::<Map<String, Json>>(&headers_json_bytes)
        .context("jwt headers were not valid json")?;

    let Json::String(alg) = headers
        .remove("alg")
        .context("'alg' was missing from jwt headers")?
    else {
        bail!("'alg' header was not a string")
    };

    let supported_algs: RequestObjectSigningAlgValuesSupported =
        wallet_metadata.get().parsing_error()?;

    if !supported_algs.0.contains(&alg) {
        bail!("request was signed with unsupported algorithm: {alg}")
    }

    let Json::String(kid) = headers
        .remove("kid")
        .context("'kid' was missing from jwt headers")?
    else {
        bail!("'kid' header was not a string")
    };

    let client_id = request_object.client_id();
    let (did, _f) = kid.split_once('#').context(format!(
        "expected a DID verification method in 'kid' header, received '{kid}'"
    ))?;

    if client_id.0 != did {
        bail!(
            "DIDs from 'kid' ({did}) and 'client_id' ({}) do not match",
            client_id.0
        )
    }

    if let Some(dids) = trusted_dids {
        if !dids.iter().any(|trusted_did| trusted_did == did) {
            bail!("'client_id' ({did}) is not in the list of trusted dids")
        }
    }

    let did = DIDBuf::from_str(did)?;

    let resolution_output = resolver
        .resolve(did.as_did())
        // .fetch_public_jwk(Some(&vm))
        .await
        .context("unable to resolve key from verification method")?;

    let key = resolution_output
        .document
        .verification_method
        .iter()
        .find(|method| method.type_ == "JsonWebSignature2020")
        .map(|method| method.properties.get("publicKeyJwk"))
        .flatten()
        .context("verification method not found in DID document")?;

    let jwk: JWK = serde_json::from_value(key.clone())
        .context("unable to parse JWK from verification method")?;

    let _: Json = ssi::claims::jwt::decode_verify(&request_jwt, &jwk)
        .context("request signature could not be verified")?;

    Ok(())
}
