/*
 * step_4_3
 *
 * No description provided (generated by Openapi Generator https://github.com/openapitools/openapi-generator)
 *
 * The version of the OpenAPI document: 0.1.0
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct RolesPeriodModel {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "permissions")]
    pub permissions: crate::models::Permissions,
    #[serde(rename = "slug")]
    pub slug: String,
}

impl RolesPeriodModel {
    pub fn new(name: String, permissions: crate::models::Permissions, slug: String) -> RolesPeriodModel {
        RolesPeriodModel {
            name,
            permissions,
            slug,
        }
    }
}


