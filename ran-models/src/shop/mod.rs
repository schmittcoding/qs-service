pub mod orders;
pub mod mileage;

use crate::errors::validate_base64_optional;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use serde::{Deserialize, Serialize};
use validator::Validate;


#[derive(Serialize, Deserialize, Debug)]
pub struct ShopMigrate {
    pub product_num: i32,
    pub item_image: Vec<u8>, // Assuming the image is returned as binary data
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShopData {
    pub product_num: i32,
    pub item_main: i32,
    pub item_sub: i32,
    pub item_name: String,
    pub item_price: f64,    // Assuming money maps to f64
    pub item_category: i16, // SmallInt maps to i16
    pub item_description: String,
    pub item_discount: Option<f64>, // Float mapped to f32
    pub item_discount_type: Option<i16>,
    pub item_image: Vec<u8>, // Assuming the image is returned as binary data
    pub item_status: i16,
    pub item_stock: i32,
    pub item_purchase_limit: Option<i32>,
}

impl ShopData {
    pub fn new(
        product_num: i32,
        item_main: i32,
        item_sub: i32,
        item_name: String,
        item_price: f64,
        item_category: i16,
        item_description: String,
        item_discount: Option<f64>,
        item_discount_type: Option<i16>,
        item_image: Vec<u8>,
        item_status: i16,
        item_stock: i32,
        item_purchase_limit: Option<i32>
    ) -> Self {
        ShopData {
            product_num,
            item_main,
            item_sub,
            item_name,
            item_price,
            item_category,
            item_description,
            item_discount,
            item_discount_type,
            item_image,
            item_status,
            item_stock,
            item_purchase_limit
        }
    }

    pub fn to_response(&self) -> Result<ShopDataResponse, base64::DecodeError> {
        Ok(ShopDataResponse {
            product_num: self.product_num,
            item_name: self.item_name.clone(),
            item_price: self.item_price,
            item_category: self.item_category,
            item_description: self.item_description.clone(),
            item_discount: self.item_discount,
            item_discount_type: self.item_discount_type,
            item_image: BASE64.encode(&self.item_image),
            item_status: self.item_status,
            item_stock: self.item_stock,
            item_purchase_limit: self.item_purchase_limit
        })
    }

    pub fn to_admin_response(&self) -> Result<AdminShopDataResponse, base64::DecodeError> {
        Ok(AdminShopDataResponse {
            product_num: self.product_num,
            item_main: self.item_main,
            item_sub: self.item_sub,
            item_name: self.item_name.clone(),
            item_price: self.item_price,
            item_category: self.item_category,
            item_description: self.item_description.clone(),
            item_discount: self.item_discount,
            item_discount_type: self.item_discount_type,
            item_image: Some(BASE64.encode(&self.item_image)),
            item_status: self.item_status,
            item_stock: self.item_stock,
            item_purchase_limit: self.item_purchase_limit
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ShopDataResponse {
    pub product_num: i32,
    pub item_name: String,
    pub item_price: f64,
    pub item_category: i16,
    pub item_description: String,
    pub item_discount: Option<f64>,
    pub item_discount_type: Option<i16>,
    pub item_image: String,
    pub item_status: i16,
    pub item_stock: i32,
    pub item_purchase_limit: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Validate, Clone)]
pub struct AdminShopDataResponse {
    pub product_num: i32,
    pub item_main: i32,
    pub item_sub: i32,
    pub item_name: String,
    pub item_price: f64,    // Assuming money maps to f64
    pub item_category: i16, // SmallInt maps to i16
    pub item_description: String,
    pub item_discount: Option<f64>, // Float mapped to f32
    pub item_discount_type: Option<i16>,
    #[validate(custom(function = "validate_base64_optional"))]
    pub item_image: Option<String>,
    pub item_status: i16,
    pub item_stock: i32,
    pub item_purchase_limit: Option<i32>,
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct AddShopData {
    pub item_main: i32,
    pub item_sub: i32,
    pub item_name: String,
    pub item_price: f64,    // Assuming money maps to f64
    pub item_category: i16, // SmallInt maps to i16
    pub item_description: String,
    pub item_discount: Option<f32>, // Float mapped to f32
    pub item_discount_type: Option<i16>,
    #[validate(custom(function = "validate_base64_optional"))]
    pub item_image: Option<String>,
    pub item_status: i16,
    pub item_stock: i32,
    pub item_purchase_limit: Option<i32>,
    pub item_purchase_limit_type: Option<i16>
}

#[derive(Serialize, Deserialize, Debug, Validate)]
pub struct EditShopData {
    pub item_main: i32,
    pub item_sub: i32,
    pub item_name: String,
    pub item_price: f64,    // Assuming money maps to f64
    pub item_category: i16, // SmallInt maps to i16
    pub item_description: String,
    pub item_discount: Option<f32>, // Float mapped to f32
    pub item_discount_type: Option<i16>,
    #[validate(custom(function = "validate_base64_optional"))]
    pub item_image: Option<String>,
    pub item_status: i16,
    pub item_stock: i32,
    pub item_purchase_limit: Option<i32>,
    pub item_purchase_limit_type: Option<i16>
}

#[derive(Serialize, Deserialize)]
pub struct PurchaseItem {
    pub product_num: i32,
    pub quantity: i32,
}

#[derive(Deserialize)]
pub struct AdminShopQuery {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub status: Option<i16>,
    #[serde(default)]
    #[serde(deserialize_with = "multiple_query_fix")]
    pub category: Vec<i16>,
}

fn multiple_query_fix<'de, D>(deserializer: D) -> Result<Vec<i16>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str_sequence = String::deserialize(deserializer)?;
    if str_sequence.is_empty() {
        return Ok(Vec::new());
    }

    str_sequence
        .split(',')
        .map(|item| item.parse::<i16>().map_err(serde::de::Error::custom))
        .collect()
}
