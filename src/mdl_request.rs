use crate::{utils::Error, jar::RequestObject};
use isomdl::definitions::device_request::{ItemsRequest, Namespaces};
use crate::{
    presentation_exchange::{
        Constraints, ConstraintsField, InputDescriptor,
        PresentationDefinition,
    },
    utils::NonEmptyVec,
};
use serde_json::json;
use isomdl::definitions::helpers::NonEmptyMap;
use std::collections::BTreeMap;
pub enum CredentialFormat {
    MDOC,
    VC,
}

pub fn prepare_mdl_request_object(requested_fields: NonEmptyMap< String, NonEmptyMap<Option<String>, Option<bool>>> , client_id: String, redirect_uri: String, presentation_id: String) -> Result<RequestObject, Error>{
    let presentation_definition = mdl_presentation_definition(requested_fields, presentation_id)?;

    Ok( RequestObject{
        iss: "".to_string(), // check value
        aud: "".to_string(),  // check value
        response_type: "vp_token".to_string(),
        client_id: client_id.clone(), // check value
        client_id_scheme: None, // check value
        redirect_uri: Some(redirect_uri),
        scope: Some("openid".to_string()),
        state:"".to_string(), 
        presentation_definition: Some(presentation_definition),
        presentation_definition_uri: None,
        client_metadata: "yes".to_string(), // check value
        client_metadata_uri: None,
        response_mode: Some("direct_post.jwt".to_string()),
        nonce: Some(client_id) // check value
    })
}

fn mdl_presentation_definition(
    namespaces: NonEmptyMap< String, NonEmptyMap<Option<String>, Option<bool>>>,
    presentation_id: String
) -> Result<PresentationDefinition, Error> {
    let input_descriptors = build_input_descriptors(namespaces);
    Ok(PresentationDefinition{
        id: presentation_id,
        input_descriptors: input_descriptors,
        name: None,
        purpose: None,
        format: None,
    })
}

//TODO: allow for specifying the algorithm
fn build_input_descriptors(namespaces: NonEmptyMap< String, NonEmptyMap<Option<String>, Option<bool>>>) -> Vec<InputDescriptor>{
    let path_base = "$.mdoc";

    let doc_type_filter = json!({
            "type": "string",
            "const": "org.iso.18013.5.1.mDL"
        });

    let input_descriptors: Vec<InputDescriptor> = namespaces.iter().map(|namespace| {
        let namespace_filter = json!({
            "type": "string",
            "const": namespace.0
        });

        let format = json!({
            "mso_mdoc": {
                "alg": [
                    "EdDSA",
                    "ES256"
                ]
            }});

        let mut fields: Vec<ConstraintsField> = vec![];

        fields.push(ConstraintsField {
            path: NonEmptyVec::new(format!("{}{}", path_base, "doc_type")),
            id: None,
            purpose: None,
            name: None,
            filter: Some(doc_type_filter.clone()),
            optional: None,
            intent_to_retain: None,
        });
    
        fields.push(ConstraintsField {
            path: NonEmptyVec::new(format!("{}{}", path_base, "namespace")),
            id: None,
            purpose: None,
            name: None,
            filter: Some(namespace_filter),
            optional: None,
            intent_to_retain: None,
        });

        let namespace_fields = namespace.1;
        let fields: Vec<ConstraintsField> =  namespace_fields.iter().map(|f| {
            ConstraintsField { 
                path: NonEmptyVec::new(format!("{}{:?}", path_base, f.0)),
                 id: None,
                 purpose:None,
                 name:None,
                filter: None,
                optional: None,
                intent_to_retain: *f.1 
            
            }
        }).collect();

        let constraints = Constraints{
            fields: Some(fields),
            limit_disclosure: None,
        };

        InputDescriptor{ 
            id: "mDL".to_string(),
            name: None,
            purpose: None,
            format: Some(format),
            constraints: Some(constraints),
            schema: None }
    }).collect();

    input_descriptors

}

impl From<Constraints> for ItemsRequest {
    fn from(constraints: Constraints) -> Self {
        let mut fields = constraints.fields.expect("there were no fields requested");

    //     let mut namespace_fields: Vec<(Option<&String>, Option<bool>)> = fields.iter().map(|f| {
    //         let x = f.path.
    //         (f.path.first().clone(), f.intent_to_retain)}).collect();
    //     namespace_fields.retain(|field| field.0.is_some());
    //     let oid4vp_namespaces: Vec<(Option<&String>, Option<bool>)>  = namespace_fields.iter().map(
    //         |field|   
    //         if field.1.is_some() {field.to_owned()}
    //         else {((field.0.clone(), Some(false)))}
    // ).collect();
        //TODO: function that matches field paths array items to valid mdl namespace fields


        let mdl_request = minimal_mdl_request_isomdl();
        let x: NonEmptyMap<String, bool> = NonEmptyMap::try_from(mdl_request).unwrap();
        let namespaces = NonEmptyMap::new("org.iso.18013.5.1".to_string(), x);

        ItemsRequest {
            doc_type  : "iso.18013.5.1.mDL".to_string(),
            namespaces,
            request_info: None,
        }

    }
}

impl From<Constraints> for Namespaces{
    fn from(constraints: Constraints) -> Self {
        todo!()
    }
}

// To create an item_request for an mdoc response, the multiple path options need to be narrowed down to valid mdl field names
fn match_field_paths(namespaces: Vec<(Option<String>, Option<bool>)>) {
    // let mdl_fields = mdl_fields();

    // mdl_fields.iter().

    // namespaces.iter().map(|field| {
    //     if let Some(x) = field.0 {
    //         match x {
    //             String { 
                    
    //             }
    //         }
    //     }
    // })
    
    
}

fn mdl_fields() -> Vec<String> {
    vec![
        "org.iso.18013.5.1.family_name".to_string(),
        "org.iso.18013.5.1.given_name".to_string(),
        "org.iso.18013.5.1.birth_date".to_string(),
        "org.iso.18013.5.1.issue_date".to_string(),
        "org.iso.18013.5.1.expiry_date".to_string(),
        "org.iso.18013.5.1.issuing_country".to_string(),
        "org.iso.18013.5.1.issuing_authority".to_string(),
        "org.iso.18013.5.1.document_number".to_string(),
        "org.iso.18013.5.1.portrait".to_string(),
        "org.iso.18013.5.1.driving_privileges".to_string(),
        "org.iso.18013.5.1.un_distinguishing_sign".to_string(),
        "org.iso.18013.5.1.administrative_number".to_string(),
        "org.iso.18013.5.1.sex".to_string(),
        "org.iso.18013.5.1.height".to_string(),
        "org.iso.18013.5.1.weight".to_string(),
        "org.iso.18013.5.1.eye_colour".to_string(),
        "org.iso.18013.5.1.hair_colour".to_string(),
        "org.iso.18013.5.1.birth_place".to_string(),
        "org.iso.18013.5.1.resident_address".to_string(),
        "org.iso.18013.5.1.portrait_capture_date".to_string(),
        "org.iso.18013.5.1.age_in_years".to_string(),
        "org.iso.18013.5.1.age_birth_year".to_string(),
        "org.iso.18013.5.1.age_over_18".to_string(),
        "org.iso.18013.5.1.age_over_21".to_string(),
        "org.iso.18013.5.1.issuing_jurisdiction".to_string(),
        "org.iso.18013.5.1.nationality".to_string(),
        "org.iso.18013.5.1.resident_city".to_string(),
        "org.iso.18013.5.1.resident_state".to_string(),
        "org.iso.18013.5.1.resident_postal_code".to_string(),
        "org.iso.18013.5.1.resident_country".to_string(),
        "org.iso.18013.5.1.aamva.domestic_driving_privileges".to_string(),
        "org.iso.18013.5.1.aamva.name_suffix".to_string(),
        "org.iso.18013.5.1.aamva.organ_donor".to_string(),
        "org.iso.18013.5.1.aamva.veteran".to_string(),
        "org.iso.18013.5.1.aamva.family_name_truncation".to_string(),
        "org.iso.18013.5.1.aamva.given_name_truncation".to_string(),
        "org.iso.18013.5.1.aamva.aka_family_name.v2".to_string(),
        "org.iso.18013.5.1.aamva.aka_given_name.v2".to_string(),
        "org.iso.18013.5.1.aamva.weight_range".to_string(),
        "org.iso.18013.5.1.aamva.race_ethnicity".to_string(),
        "org.iso.18013.5.1.aamva.EDL_credential".to_string(),
        "org.iso.18013.5.1.aamva.DHS_compliance".to_string(),
        "org.iso.18013.5.1.aamva.sex".to_string(),
        "org.iso.18013.5.1.aamva.resident_county".to_string(),
        "org.iso.18013.5.1.aamva.hazmat_endorsement_expiration_date".to_string(),
        "org.iso.18013.5.1.aamva.CDL_indicator".to_string(),
        "org.iso.18013.5.1.aamva.DHS_compliance_text".to_string(),
        "org.iso.18013.5.1.aamva.DHS_temporary_lawful_status".to_string(),
        ]
}

fn minimal_mdl_request_isomdl() -> BTreeMap<String, bool> {
    BTreeMap::from([
        ("org.iso.18013.5.1.family_name".to_string(), false),
        ("org.iso.18013.5.1.given_name".to_string(), false),
        ("org.iso.18013.5.1.birth_date".to_string(), false),
        ("org.iso.18013.5.1.issue_date".to_string(), false),
        ("org.iso.18013.5.1.expiry_date".to_string(), false),
        ("org.iso.18013.5.1.issuing_country".to_string(), false),
        ("org.iso.18013.5.1.issuing_authority".to_string(), false),
        ("org.iso.18013.5.1.document_number".to_string(), false),
        ("org.iso.18013.5.1.portrait".to_string(), false),
        ("org.iso.18013.5.1.driving_privileges".to_string(), false),
        ("org.iso.18013.5.1.un_distinguishing_sign".to_string(), false),
        ("org.iso.18013.5.1.administrative_number".to_string(), false),
        ("org.iso.18013.5.1.sex".to_string(), false),
        ("org.iso.18013.5.1.height".to_string(), false),
        ("org.iso.18013.5.1.weight".to_string(), false),
        ("org.iso.18013.5.1.eye_colour".to_string(), false),
        ("org.iso.18013.5.1.hair_colour".to_string(), false),
        ("org.iso.18013.5.1.birth_place".to_string(), false),
        ("org.iso.18013.5.1.resident_address".to_string(), false),
        ("org.iso.18013.5.1.portrait_capture_date".to_string(), false),
        ("org.iso.18013.5.1.age_in_years".to_string(), false),
        ("org.iso.18013.5.1.age_birth_year".to_string(), false),
        ("org.iso.18013.5.1.age_over_18".to_string(), true,),
        ("org.iso.18013.5.1.age_over_21".to_string(), true,),
        ("org.iso.18013.5.1.issuing_jurisdiction".to_string(), false),
        ("org.iso.18013.5.1.nationality".to_string(), false),
        ("org.iso.18013.5.1.resident_city".to_string(), false),
        ("org.iso.18013.5.1.resident_state".to_string(), false),
        ("org.iso.18013.5.1.resident_postal_code".to_string(), false),
        ("org.iso.18013.5.1.resident_country".to_string(), false),
        
        ])
}

fn aamva_isomdl_data() -> BTreeMap<String, bool> {
    BTreeMap::from([
        ("domestic_driving_privileges".to_string(), false),
        ("name_suffix".to_string(), false),
        ("organ_donor".to_string(), false),
        ("veteran".to_string(), false),
        ("family_name_truncation".to_string(), false),
        ("given_name_truncation".to_string(), false),
        ("aka_family_name.v2".to_string(), false),
        ("aka_given_name.v2".to_string(), false),
        ("weight_range".to_string(), false),
        ("race_ethnicity".to_string(), false),
        ("EDL_credential".to_string(), false),
        ("DHS_compliance".to_string(), false),
        ("sex".to_string(), false),
        ("resident_county".to_string(), false),
        ("hazmat_endorsement_expiration_date".to_string(), false),
        ("CDL_indicator".to_string(), false),
        ("DHS_compliance_text".to_string(), false),
        ("DHS_temporary_lawful_status".to_string(), false),
        ])
}

pub fn minimal_mdl_request() -> BTreeMap<Option<String>, Option<bool>> {
    BTreeMap::from([
        (Some("org.iso.18013.5.1.family_name".to_string()),Some( true)),
        (Some("org.iso.18013.5.1.given_name".to_string()),Some( true)),
        (Some("org.iso.18013.5.1.birth_date".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.issue_date".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.expiry_date".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.issuing_country".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.issuing_authority".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.document_number".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.portrait".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.driving_privileges".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.un_distinguishing_sign".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.administrative_number".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.sex".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.height".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.weight".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.eye_colour".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.hair_colour".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.birth_place".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.resident_address".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.portrait_capture_date".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.age_in_years".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.age_birth_year".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.age_over_18".to_string()), Some(true,)),
        (Some("org.iso.18013.5.1.age_over_21".to_string()), Some(false,)),
        (Some("org.iso.18013.5.1.issuing_jurisdiction".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.nationality".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.resident_city".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.resident_state".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.resident_postal_code".to_string()), Some(false)),
        (Some("org.iso.18013.5.1.resident_country".to_string()), Some(false)),
        
        ])
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn request_example() {
        let minimal_mdl_request = NonEmptyMap::try_from(minimal_mdl_request()).unwrap();
        let namespaces = NonEmptyMap::new("org.iso.18013.5.1".to_string(), minimal_mdl_request);
        let client_id = "nonce".to_string();
        let redirect_uri = "localhost::3000".to_string();
        let presentation_id = "test minimal mdl request".to_string();

        let request_object = prepare_mdl_request_object(namespaces, client_id, redirect_uri, presentation_id).unwrap();

        println!("request object: {:?}", request_object);

    }
}