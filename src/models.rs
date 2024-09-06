use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetadataItem {
    #[serde(rename = "type")]
    pub typ: String,
    pub has_content_id: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_xvc: Option<bool>,
    pub content_id: String,
    pub product_id: String,
    pub package_family_name: String,
    pub one_store_product_id: String,
    pub version: String,
    pub size: usize,
    pub allowed_product_id: String,
    pub allowed_package_family_name: String,
    pub path: String,
    pub availability: String,
    pub generation: String,
    pub related_media: Vec<String>,
    pub related_media_family_names: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub items: Vec<MetadataItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_metadata() {
        let json = r#"
        {
            "items": [
                {
                    "type": "app",
                    "hasContentId": false,
                    "contentId": "",
                    "productId": "",
                    "packageFamilyName": "11032Reconco.XboxControllerTester_thvmwcgtjwwvy",
                    "oneStoreProductId": "9NBLGGH4PNC7",
                    "version": "0",
                    "size": 0,
                    "allowedProductId": "",
                    "allowedPackageFamilyName": "",
                    "path": "/col/content/%7BA89ECE52-7E8E-444F-BBD0-C68B76C2ECA4%7D%2311032Reconco.XboxControllerTester_thvmwcgtjwwvy",
                    "availability": "available",
                    "generation": "uwpgen9",
                    "relatedMedia": [],
                    "relatedMediaFamilyNames": []
                }
            ]
        }
        "#;

        let deserialized = serde_json::from_str::<Metadata>(json)
            .expect("Failed deserializing");
    
        let first = deserialized.items.first().expect("Failed getting first entry");
        assert_eq!(first.typ, "app");
        assert!(!first.has_content_id);
        assert_eq!(first.content_id, "");
        assert_eq!(first.product_id, "");
        assert_eq!(first.package_family_name, "11032Reconco.XboxControllerTester_thvmwcgtjwwvy");
        assert_eq!(first.one_store_product_id, "9NBLGGH4PNC7");
        assert_eq!(first.version, "0");
        assert_eq!(first.size, 0);
        assert_eq!(first.allowed_product_id, "");
        assert_eq!(first.allowed_package_family_name, "");
        assert_eq!(first.path, "/col/content/%7BA89ECE52-7E8E-444F-BBD0-C68B76C2ECA4%7D%2311032Reconco.XboxControllerTester_thvmwcgtjwwvy");
        assert_eq!(first.availability, "available");
        assert_eq!(first.generation, "uwpgen9");
        assert!(first.related_media.is_empty());
        assert!(first.related_media_family_names.is_empty());
    }
}