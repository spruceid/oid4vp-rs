pub use crate::core::authorization_request::parameters::State;
use crate::core::object::TypedParameter;
use crate::core::presentation_submission::PresentationSubmission as PresentationSubmissionParsed;

use anyhow::{bail, Error};
use base64::prelude::*;
use serde_json::{Map, Value as Json};
use ssi::{claims::vc, prelude::AnyJsonPresentation};

#[derive(Debug, Clone)]
pub struct IdToken(pub String);

impl TypedParameter for IdToken {
    const KEY: &'static str = "id_token";
}

impl TryFrom<Json> for IdToken {
    type Error = Error;

    fn try_from(value: Json) -> Result<Self, Self::Error> {
        serde_json::from_value(value).map(Self).map_err(Into::into)
    }
}

impl From<IdToken> for Json {
    fn from(value: IdToken) -> Self {
        value.0.into()
    }
}

/// OpenID Connect for Verifiable Presentations specification defines `vp_token` parameter:
///
/// > JSON String or JSON object that MUST contain a single Verifiable Presentation or
/// > an array of JSON Strings and JSON objects each of them containing a Verifiable Presentations.
/// >
/// > Each Verifiable Presentation MUST be represented as a JSON string (that is a Base64url encoded value)
/// > or a JSON object depending on a format as defined in Appendix A of [OpenID.VCI].
/// >
/// > If Appendix A of [OpenID.VCI] defines a rule for encoding the respective Credential
/// > format in the Credential Response, this rules MUST also be followed when encoding Credentials of
/// > this format in the vp_token response parameter. Otherwise, this specification does not require
/// > any additional encoding when a Credential format is already represented as a JSON object or a JSON string.
///
/// See: [OpenID.VP#section-6.1-2.2](https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-6.1-2.2)
#[derive(Debug, Clone)]
pub enum VpToken {
    Single(Vec<u8>),
    SingleAsMap(Map<String, Json>),
    Many(Vec<VpToken>),
}

impl TypedParameter for VpToken {
    const KEY: &'static str = "vp_token";
}

impl TryFrom<Json> for VpToken {
    type Error = Error;

    fn try_from(value: Json) -> Result<Self, Self::Error> {
        match value {
            // NOTE: When parsing a Json string, it must be base64Url encoded,
            // therefore the base64 encoded string is decoded for internal representation
            // of the VP token.
            Json::String(s) => Ok(Self::Single(BASE64_URL_SAFE_NO_PAD.decode(s)?)),
            // NOTE: When the Json is an object, it must be a map.
            Json::Object(map) => Ok(Self::SingleAsMap(map)),
            Json::Array(arr) => arr
                .into_iter()
                .map(Self::try_from)
                .collect::<Result<Vec<Self>, Self::Error>>()
                .map(Self::Many),
            _ => Err(Error::msg("Invalid vp_token")),
        }
    }
}

impl From<VpToken> for Json {
    fn from(value: VpToken) -> Self {
        match value {
            VpToken::Single(s) => serde_json::Value::String(BASE64_URL_SAFE_NO_PAD.encode(s)),
            VpToken::SingleAsMap(map) => serde_json::Value::Object(map),
            VpToken::Many(tokens) => Self::Array(tokens.into_iter().map(Self::from).collect()),
        }
    }
}

fn extract_object(json: Json) -> Result<Map<String, Json>, Error> {
    match json {
        Json::Object(m) => Ok(m),
        _ => bail!("expected JSON object"),
    }
}

impl TryFrom<vc::v1::syntax::JsonPresentation> for VpToken {
    type Error = Error;

    fn try_from(vp: vc::v1::syntax::JsonPresentation) -> Result<Self, Self::Error> {
        Ok(VpToken::SingleAsMap(extract_object(serde_json::to_value(
            vp,
        )?)?))
    }
}

impl TryFrom<vc::v2::syntax::JsonPresentation> for VpToken {
    type Error = Error;

    fn try_from(vp: vc::v2::syntax::JsonPresentation) -> Result<Self, Self::Error> {
        Ok(VpToken::SingleAsMap(extract_object(serde_json::to_value(
            vp,
        )?)?))
    }
}

impl TryFrom<AnyJsonPresentation> for VpToken {
    type Error = Error;

    fn try_from(vp: AnyJsonPresentation) -> Result<Self, Self::Error> {
        Ok(VpToken::SingleAsMap(extract_object(serde_json::to_value(
            vp,
        )?)?))
    }
}

#[derive(Debug, Clone)]
pub struct PresentationSubmission {
    raw: Json,
    parsed: PresentationSubmissionParsed,
}

impl PresentationSubmission {
    pub fn into_parsed(self) -> PresentationSubmissionParsed {
        self.parsed
    }

    pub fn parsed(&self) -> &PresentationSubmissionParsed {
        &self.parsed
    }
}

impl TryFrom<PresentationSubmissionParsed> for PresentationSubmission {
    type Error = Error;

    fn try_from(parsed: PresentationSubmissionParsed) -> Result<Self, Self::Error> {
        let raw = serde_json::to_value(parsed.clone())?;
        Ok(Self { raw, parsed })
    }
}

impl TypedParameter for PresentationSubmission {
    const KEY: &'static str = "presentation_submission";
}

impl TryFrom<Json> for PresentationSubmission {
    type Error = Error;

    fn try_from(raw: Json) -> Result<Self, Self::Error> {
        let parsed = serde_json::from_value(raw.clone())?;
        Ok(Self { raw, parsed })
    }
}

impl From<PresentationSubmission> for Json {
    fn from(value: PresentationSubmission) -> Self {
        value.raw
    }
}
