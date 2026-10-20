use serde::{Deserialize, Serialize};
use tiberius::Row;
use uuid::Uuid;

#[derive(Debug)]
pub enum ShopDiscountType {
    Percentage = 1,
    Fixed = 2,
}

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
pub struct ItemShopBase {
    pub item_main:              i32,
    pub item_sub:               i32,
    pub item_name:              String,
    pub item_price:             f64,
    pub item_category:          i16,
    pub item_description:       String,
    pub item_discount:          Option<f64>,
    pub item_discount_type:     Option<i16>,
    pub item_status:            i16,
    pub item_stock:             i32,
    pub item_purchase_limit:    Option<i32>,
    pub item_purchase_limit_type: Option<i16>,
    pub item_tag:               String,
    pub shop_type:              u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateItemShopRequest {
    pub item_image:             Option<String>,
    pub removed_gallery_ids:    Option<Vec<Uuid>>,
    #[serde(flatten)]
    pub base:                   ItemShopBase,
}

#[derive(Serialize, Deserialize, Debug, Clone, utoipa::ToSchema)]
pub struct ItemGalleryImage {
    pub image_id:   Uuid,
    pub image_url:  String,
    pub sort_order: i16,
}

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct ItemShopData {
    pub product_num:            i32,
    pub item_image:             String,
    pub item_gallery:           Vec<ItemGalleryImage>,
    #[serde(flatten)]
    pub base:                   ItemShopBase,
}

#[derive(Serialize, Deserialize, Debug, utoipa::ToSchema)]
pub struct GetItemShopData {
    pub discounted_price:       f64,
    pub purchase_limit_text:    String,
    #[serde(flatten)]
    pub base:                   ItemShopData,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetItemShopResponse {
    pub product_num:                i32,
    pub item_image:                 String,
    pub item_gallery:               Vec<ItemGalleryImage>,
    #[serde(flatten)]
    pub base:                       ItemShopBase,
    pub remaining_purchase_limit:   Option<i32>,

    pub final_price:                f64,
    pub discount_percent:           f64,

    pub purchase_limit_text:        String,

    pub mileage_points_to_earn:     f64,
    pub current_mileage_rate:       f64,
}



impl ShopDiscountType {
    pub fn from_i16(value: i16) -> Option<Self> {
        match value {
            1 => Some(Self::Percentage),
            2 => Some(Self::Fixed),
            _ => None,
        }
    }
}

impl ItemShopBase {
    pub fn from_row(row: &Row) -> Self {
        Self {
            item_category: row.get::<i16, _>("ItemCategory").unwrap_or_default(),
            item_description: row
                .get::<&str, _>("ItemDescription")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            item_discount: row
                .try_get::<f64, _>("ItemDiscount")
                .unwrap_or_default(),
            item_discount_type: row.try_get::<i16, _>("ItemDiscountType").unwrap_or_default(),
            item_main: row.get::<i32, _>("ItemMain").unwrap_or_default(),
            item_name: row
                .get::<&str, _>("ItemName")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            item_price: row
                .try_get::<f64, _>("ItemPrice")
                .ok()
                .flatten()
                .unwrap_or_default(),
            item_purchase_limit: row.try_get::<i32, _>("PurchaseLimitPerAccount").unwrap_or_default(),
            item_purchase_limit_type: row.try_get::<i16, _>("PurchaseLimitType").unwrap_or_default(),
            item_status: row.get::<i16, _>("ItemStatus").unwrap_or_default(),
            item_stock: row.get::<i32, _>("ItemStock").unwrap_or_default(),
            item_sub: row.get::<i32, _>("ItemSub").unwrap_or_default(),
            item_tag: row
                .get::<&str, _>("ItemTag")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            shop_type: row.get::<u8, _>("ShopType").unwrap_or(1)
        }
    }
}

impl ItemGalleryImage {
    pub fn from_row(row: &Row) -> Self {
        let image_id = row.get::<Uuid, _>("ImageID").unwrap_or_default();

        Self {
            image_id,
            image_url: row
                .get::<&str, _>("ImageUrl")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            sort_order: row.get::<i16, _>("SortOrder").unwrap_or_default(),
        }
    }
}

impl ItemShopData {
    pub fn from_row(row: &Row) -> Self {
        Self {
            item_image: row
                .get::<&str, _>("NewItemImage")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            product_num: row.get::<i32, _>("ProductNum").unwrap_or_default(),
            item_gallery: Vec::new(),
            base: ItemShopBase::from_row(row)
        }
    }
}

impl GetItemShopData {
    pub fn from_row(row: &Row) -> Self {
        Self {
            discounted_price: row
                .try_get::<f64, _>("DiscountedPrice")
                .ok()
                .flatten()
                .unwrap_or_default(),
            purchase_limit_text: row
                .get::<&str, _>("LimitTypeText")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            base: ItemShopData::from_row(row)
        }
    }
}

impl GetItemShopResponse {
    pub fn from_row(row: &Row) -> Self {
        let base = ItemShopBase::from_row(row);

        let final_price = calculate_final_price(base.item_price, base.item_discount, base.item_discount_type);
        let discount_percent = calculate_discount_percent(base.item_price, base.item_discount, base.item_discount_type);

        Self {
            base,
            discount_percent,
            final_price,
            item_gallery: Vec::new(),
            item_image: row
                .get::<&str, _>("NewItemImage")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            product_num: row.get::<i32, _>("ProductNum").unwrap_or_default(),

            remaining_purchase_limit: row.try_get::<i32, _>("RemainingPurchaseLimit").unwrap_or_default(),
            purchase_limit_text: row
                .get::<&str, _>("LimitTypeText")
                .map_or_else(|| "".to_string(), |s| s.to_string()),
            mileage_points_to_earn: row
                .try_get::<f64, _>("MileagePointsToEarn")
                .ok()
                .flatten()
                .unwrap_or_default(),
            current_mileage_rate: row
                .try_get::<f64, _>("CurrentMileageRate")
                .ok()
                .flatten()
                .unwrap_or_default(),
        }
    }
}


pub fn calculate_final_price(
    original_price: f64,
    discount: Option<f64>,
    discount_type: Option<i16>,
) -> f64 {
    let Some(discount) = discount else { return original_price };
    let Some(d_type) = discount_type else { return original_price };

    match ShopDiscountType::from_i16(d_type) {
        Some(ShopDiscountType::Fixed) => original_price - discount,
        Some(ShopDiscountType::Percentage) => original_price - (original_price * (discount / 100.0)),
        None => original_price,
    }
}

pub fn calculate_discount_percent(
    original_price: f64,
    discount: Option<f64>,
    discount_type: Option<i16>,
) -> f64 {
    let Some(discount) = discount else { return 0.0 };
    let Some(d_type) = discount_type else { return 0.0 };

    match ShopDiscountType::from_i16(d_type) {
        Some(ShopDiscountType::Percentage) => discount,
        Some(ShopDiscountType::Fixed) => (discount / original_price) * 100.0,
        None => 0.0,
    }
}
