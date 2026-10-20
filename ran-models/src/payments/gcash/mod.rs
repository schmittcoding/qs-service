use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema)]
pub struct GCashTransaction {
    pub reference_number: String,
    pub proof_image: Option<String>,
}
