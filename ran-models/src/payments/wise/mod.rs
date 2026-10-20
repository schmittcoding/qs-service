use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, utoipa::ToSchema)]
pub struct WiseTransaction {
    pub reference_number: String,
}
