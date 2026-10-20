use super::WebDB;
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::{StreamExt};
use ran_models::{payments::{
    gcash::GCashTransaction, AddGatewayInfo, AddRechargeInfo, GatewayInfo, RechargeInfo,
    Transaction, TransactionInfo,
}, BaseDataCountResult};
use tiberius::Row;
use uuid::Uuid;

fn parse_recharge_info(row: &Row) -> Option<RechargeInfo> {
    Some(RechargeInfo {
        id: row.try_get::<Uuid, _>("RechargeID").unwrap()?,
        r#type: row.get("RechargeType")?,
        amount: row.get("RechargeAmount")?,
        discount: row.try_get("RechargeDiscount").unwrap(),
        points: row.get("RechargePoints")?,
        status: row.get("RechargeStatus")?,
    })
}

fn parse_gateway_info(row: &Row) -> Option<GatewayInfo> {
    let image = BASE64.encode(row.try_get::<&[u8], _>("GatewayImage").ok().flatten()?);
    Some(GatewayInfo {
        id: row.get("GatewayID")?,
        provider: row.get("GatewayProvider")?,
        name: row
            .try_get::<&str, _>("GatewayName")
            .ok()
            .flatten()
            .unwrap()
            .to_string(),
        number: row
            .try_get::<&str, _>("GatewayNumber")
            .ok()
            .flatten()
            .unwrap()
            .to_string(),
        image,
        status: row.get("GatewayStatus")?,
    })
}

#[async_trait::async_trait]
pub trait WebTransactionTrait {
    async fn execute_query(&self, query: &str, params: &[&dyn tiberius::ToSql]) -> Result<()>;
    async fn transact(&self, transaction: Transaction) -> Result<Uuid>;
    async fn get_recharge_info(&self, status: Option<u8>, r#type: Option<i16>) -> Result<Vec<RechargeInfo>>;
    async fn get_recharge_info_by_id(&self, recharge_id: Uuid) -> Result<RechargeInfo>;
    async fn get_gateway_info(&self) -> Result<Vec<GatewayInfo>>;
    async fn get_gateway_info_by_id(&self, gateway_id: Uuid) -> Result<GatewayInfo>;
    async fn add_gateway_info(&self, data: AddGatewayInfo) -> Result<()>;
    async fn get_recharge_logs(&self, user_num: i64) -> Result<Vec<Row>>;
    async fn update_transaction_status(&self, transaction_id: Uuid, status: i16) -> Result<i64>;
    async fn get_recharge_logs_filtered(&self, status: i16) -> Result<Vec<Row>>;
    async fn update_recharge_info_by_id(
        &self,
        recharge_id: Uuid,
        data: AddRechargeInfo,
    ) -> Result<RechargeInfo>;
    async fn get_recharge_logs_all(&self) -> Result<Vec<Row>>;
    async fn update_gateway_info_by_id(
        &self,
        gateway_id: Uuid,
        data: AddGatewayInfo,
    ) -> Result<GatewayInfo>;
    async fn transact_gcash(
        &self,
        transaction: Transaction,
        gcash_info: GCashTransaction,
    ) -> Result<Uuid>;
    async fn transact_v2(
        &self,
        transaction: Transaction,
        info: TransactionInfo,
    ) -> Result<Uuid>;

    async fn get_transactions(
        &self,
        page: i16,
        page_size: i16,
        r#type: Option<Vec<i16>>,
        status: Option<Vec<i16>>,
        search: Option<String>
    ) -> Result<BaseDataCountResult>;
}

#[async_trait::async_trait]
impl WebTransactionTrait for WebDB {
    async fn update_transaction_status(&self, transaction_id: Uuid, status: i16) -> Result<i64> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        // Fetch the UserNum before updating (table has triggers, so OUTPUT clause is not allowed)
        let select_query = "SELECT UserNum FROM WebTransaction WHERE TransactionID = @P1 AND TransactionStatus = 0";
        let row = client
            .query(select_query, &[&transaction_id])
            .await
            .context("Failed to fetch transaction")?
            .into_row()
            .await
            .context("Failed to read transaction row")?
            .ok_or_else(|| anyhow::anyhow!("Transaction with id {} not found", transaction_id))?;

        let user_num: i64 = row.get("UserNum").ok_or_else(|| {
            anyhow::anyhow!("UserNum not found in transaction row")
        })?;

        // Now perform the update
        let update_query = "UPDATE WebTransaction SET TransactionStatus = @P1 WHERE TransactionID = @P2 AND TransactionStatus = 0";
        let rows_updated = client
            .execute(update_query, &[&status, &transaction_id])
            .await
            .context("Failed to execute update")?;

        if rows_updated.total() == 0 {
            return Err(anyhow::anyhow!(
                "Transaction with id {} not found",
                transaction_id
            ));
        }

        Ok(user_num)
    }

    async fn get_recharge_logs(&self, user_num: i64) -> Result<Vec<Row>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query =
            "SELECT * FROM WebTransaction WHERE UserNum = @P1 ORDER BY TransactionTime DESC;";
        let mut stream = client
            .query(query, &[&user_num])
            .await
            .context("Failed to execute query")?;

        let mut transactions: Vec<Row> = Vec::new();

        while let Some(qir) = stream.next().await {
            match qir {
                Ok(qi) => {
                    let qi_owned = qi.into_row();
                    if let Some(row_data) = qi_owned {
                        let transaction = row_data;
                        transactions.push(transaction);
                    }
                }
                Err(e) => {
                    log::error!("❌ Error reading row: {:?}", e);
                }
            }
        }

        Ok(transactions) // ✅ Return the collected list
    }

    async fn get_recharge_logs_all(&self) -> Result<Vec<Row>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "SELECT * FROM WebTransaction ORDER BY TransactionTime DESC";
        let mut stream = client
            .query(query, &[])
            .await
            .context("Failed to execute query")?;

        let mut transactions: Vec<Row> = Vec::new();

        while let Some(qir) = stream.next().await {
            match qir {
                Ok(qi) => {
                    let qi_owned = qi.into_row();
                    if let Some(row_data) = qi_owned {
                        let transaction = row_data;
                        transactions.push(transaction);
                    }
                }
                Err(e) => {
                    log::error!("❌ Error reading row: {:?}", e);
                }
            }
        }

        Ok(transactions) // ✅ Return the collected list
    }

    async fn get_recharge_logs_filtered(&self, status: i16) -> Result<Vec<Row>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "SELECT * FROM WebTransaction WHERE TransactionStatus = @P1 ORDER BY TransactionTime DESC";
        let mut stream = client
            .query(query, &[&status])
            .await
            .context("Failed to execute query")?;

        let mut transactions: Vec<Row> = Vec::new();

        while let Some(qir) = stream.next().await {
            match qir {
                Ok(qi) => {
                    let qi_owned = qi.into_row();
                    if let Some(row_data) = qi_owned {
                        let transaction = row_data;
                        transactions.push(transaction);
                    }
                }
                Err(e) => {
                    log::error!("❌ Error reading row: {:?}", e);
                }
            }
        }

        Ok(transactions) // ✅ Return the collected list
    }

    async fn get_transactions(
        &self,
        page: i16,
        page_size: i16,
        r#type: Option<Vec<i16>>,
        status: Option<Vec<i16>>,
        search: Option<String>
    ) -> Result<BaseDataCountResult> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let type_list = r#type.as_ref().filter(|v| !v.is_empty()).map(|v| {
            v.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(",")
        });

        let status_list = status.as_ref().filter(|v| !v.is_empty()).map(|v| {
            v.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(",")
        });

        let sp_query = "EXEC sp_GetWebTransactions @Search = @P1, @TransactionTypeList = @P2, @TransactionStatusList = @P3, @Page = @P4, @PageSize = @P5;";

        let results = client
            .query(sp_query, &[
                &search,
                &type_list,
                &status_list,
                &page,
                &page_size
            ])
            .await?
            .into_results()
            .await?;

        let mut result_sets = results.into_iter();

        let total_count_rows = result_sets.next().unwrap_or_default();
        let total_records = total_count_rows
            .first()
            .and_then(|row| row.get::<i32, _>("TotalRecords"))
            .unwrap_or(0);

        let transactions = result_sets.next().unwrap_or_default();

        Ok(BaseDataCountResult { data: transactions, total_records })
    }

    async fn add_gateway_info(&self, data: AddGatewayInfo) -> Result<()> {
        let number = data.number.parse::<i64>().unwrap();

        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "INSERT INTO [dbo].[GatewayInfo]
           ([GatewayProvider]
           ,[GatewayName]
           ,[GatewayNumber]
           ,[GatewayImage]
           ,[GatewayStatus])
           VALUES
           (@P1,@P2,@P3,@P4,@P5);";

        let image = BASE64.decode(&data.image).unwrap();
        client
            .execute(
                query,
                &[&data.provider, &data.name, &number, &image, &data.status],
            )
            .await
            .context("Failed to insert gateway info")?;

        Ok(())
    }

    async fn get_gateway_info(&self) -> Result<Vec<GatewayInfo>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "SELECT * FROM GatewayInfo";
        let mut stream = client
            .query(query, &[])
            .await
            .context("Failed to execute query")?;

        let mut gateways = Vec::new();

        while let Some(row_result) = stream.next().await {
            match row_result {
                Ok(row) => {
                    if let Some(row_data) = &row.as_row() {
                        if let Some(gateway) = parse_gateway_info(row_data) {
                            gateways.push(gateway);
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ Error reading row: {:?}", e);
                }
            }
        }

        Ok(gateways) // ✅ Return the collected list
    }

    async fn get_gateway_info_by_id(&self, gateway_id: Uuid) -> Result<GatewayInfo> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "SELECT * FROM GatewayInfo WHERE GatewayID = @P1";
        let mut stream = client
            .query(query, &[&gateway_id])
            .await
            .context("Failed to execute query")?;

        while let Some(row_result) = stream.next().await {
            match row_result {
                Ok(row) => {
                    if let Some(row_data) = row.as_row() {
                        if let Some(gateway) = parse_gateway_info(row_data) {
                            return Ok(gateway);
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ Error reading row: {:?}", e);
                    return Err(e.into());
                }
            }
        }

        Err(anyhow::anyhow!("Gateway with id {} not found", gateway_id))
    }

    async fn get_recharge_info(&self, status: Option<u8>, r#type: Option<i16>) -> Result<Vec<RechargeInfo>> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = r#"
            SELECT RechargeID, RechargeType, RechargeAmount, RechargeDiscount, RechargePoints, RechargeStatus
            FROM RechargeInfo
            WHERE
               (@P1 IS NULL OR RechargeStatus = @P1) AND
               (@P2 IS NULL OR RechargeType = @P2)
        "#;

        let mut stream = client
            .query(query, &[
                &status,
                &r#type,
            ])
            .await
            .context("Failed to execute query")?;

        let mut recharge_list = Vec::new();

        while let Some(row_result) = stream.next().await {
            match row_result {
                Ok(row) => {
                    if let Some(row_data) = &row.as_row() {
                        if let Some(recharge) = parse_recharge_info(row_data) {
                            recharge_list.push(recharge);
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ Error reading row: {:?}", e);
                }
            }
        }

        Ok(recharge_list) // ✅ Return the collected list
    }

    async fn update_recharge_info_by_id(
        &self,
        recharge_id: Uuid,
        data: AddRechargeInfo,
    ) -> Result<RechargeInfo> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = r#"
            UPDATE RechargeInfo
            SET
                RechargeType = @P2,
                RechargeAmount = @P3,
                RechargePoints = @P4,
                RechargeStatus = @P5
            OUTPUT inserted.*
            WHERE RechargeID = @P1
        "#;

        let mut stream = client
            .query(
                query,
                &[
                    &recharge_id,
                    &data.r#type,
                    &data.amount,
                    &data.points,
                    &data.status,
                ],
            )
            .await
            .context("Failed to execute query")?;

        while let Some(row_result) = stream.next().await {
            match row_result {
                Ok(row) => {
                    if let Some(row_data) = row.into_row() {
                        if let Some(recharge) = parse_recharge_info(&row_data) {
                            return Ok(recharge);
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ Error reading row: {:?}", e);
                    return Err(e.into());
                }
            }
        }

        Err(anyhow::anyhow!(
            "Recharge with id {} not found",
            recharge_id
        ))
    }

    async fn update_gateway_info_by_id(
        &self,
        gateway_id: Uuid,
        data: AddGatewayInfo,
    ) -> Result<GatewayInfo> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;
        let image = BASE64.decode(data.image).unwrap();
        let query = r#"
            UPDATE GatewayInfo
            SET
                GatewayProvider = @P2,
                GatewayName = @P3,
                GatewayNumber = @P4,
                GatewayImage = @P5,
                GatewayStatus = @P6
            OUTPUT inserted.*
            WHERE GatewayID = @P1
        "#;

        let mut stream = client
            .query(
                query,
                &[
                    &gateway_id,
                    &data.provider,
                    &data.name,
                    &data.number,
                    &image,
                    &data.status,
                ],
            )
            .await
            .context("Failed to execute query")?;

        while let Some(row_result) = stream.next().await {
            match row_result {
                Ok(row) => {
                    if let Some(row_data) = row.into_row() {
                        if let Some(recharge) = parse_gateway_info(&row_data) {
                            return Ok(recharge);
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ Error reading row: {:?}", e);
                    return Err(e.into());
                }
            }
        }

        Err(anyhow::anyhow!("Gateway with id {} not found", gateway_id))
    }

    async fn get_recharge_info_by_id(&self, recharge_id: Uuid) -> Result<RechargeInfo> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;

        let query = "SELECT * FROM RechargeInfo WHERE RechargeID = @P1";
        let mut stream = client
            .query(query, &[&recharge_id])
            .await
            .context("Failed to execute query")?;

        while let Some(row_result) = stream.next().await {
            match row_result {
                Ok(row) => {
                    if let Some(row_data) = row.as_row() {
                        if let Some(recharge) = parse_recharge_info(row_data) {
                            return Ok(recharge);
                        }
                    }
                }
                Err(e) => {
                    log::error!("❌ Error reading row: {:?}", e);
                    return Err(e.into());
                }
            }
        }

        Err(anyhow::anyhow!("Gateway with id {} not found", recharge_id))
    }

    async fn execute_query(&self, query: &str, params: &[&dyn tiberius::ToSql]) -> Result<()> {
        let mut client = self
            .pool
            .get()
            .await
            .context("Failed to get DB connection")?;
        client.execute(query, params).await?;
        Ok(())
    }

    async fn transact_v2(
        &self,
        transaction: Transaction,
        info: TransactionInfo,
    ) -> Result<Uuid> {
        let transaction_id = Uuid::new_v4();
        let reference_number = info.reference_number();
        let maybe_proof;

        let (query_str, params) = if let Some(proof_image) = info.proof_image() {
            let decoded_proof = BASE64.decode(&proof_image)?;
            maybe_proof = Some(decoded_proof);
            (
                "
                 INSERT INTO [dbo].[WebTransaction]
                 (TransactionID, UserNum, UserName, TransactionType, TransactionAmount, TransactionPrice, TransactionCost, RefferenceCode, ProofImage)
                 VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9);
                 ",
                vec![
                    &transaction_id as &dyn tiberius::ToSql,
                    &transaction.user_num,
                    &transaction.user_name,
                    &transaction.r#type,
                    &transaction.points,
                    &transaction.amount,
                    &transaction.amount,
                    &reference_number,
                    &maybe_proof
                ],
            )
        } else {
            (
                "
                 INSERT INTO [dbo].[WebTransaction]
                 (TransactionID,UserNum, UserName, TransactionType, TransactionAmount, TransactionPrice, TransactionCost, RefferenceCode)
                 VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8);
                 ",
                vec![
                    &transaction_id as &dyn tiberius::ToSql,
                    &transaction.user_num,
                    &transaction.user_name,
                    &transaction.r#type,
                    &transaction.points,
                    &transaction.amount,
                    &transaction.amount,
                    &reference_number,
                ],
            )
        };
        self.execute_query(query_str, params.as_slice()).await?;
        Ok(transaction_id)
    }

    async fn transact_gcash(
        &self,
        transaction: Transaction,
        gcash_info: GCashTransaction,
    ) -> Result<Uuid> {
        // let mut client = self
        //     .pool
        //     .get()
        //     .await
        //     .context("Failed to get DB connection")?;
        // let check_sql = r#"
        // SELECT TOP 1 RefferenceCode
        //   FROM [dbo].[WebTransaction]
        //  WHERE RefferenceCode = @P1;
        // "#;
        // // run the query—if we get ANY row back, that reference already exists
        // let mut stream = client
        //     .query(check_sql, &[&gcash_info.reference_number])
        //     .await?;

        // if stream.try_next().await?.is_some() {
        //     bail!(
        //         "Reference code '{}' already exists",
        //         gcash_info.reference_number
        //     );
        // }

        let transaction_id = Uuid::new_v4();
        let maybe_proof;
        let (query_str, params) = if let Some(proof_image) = gcash_info.proof_image {
            let decoded_proof = BASE64.decode(&proof_image)?;
            maybe_proof = Some(decoded_proof);
            (
                "
                 INSERT INTO [dbo].[WebTransaction]
                 (TransactionID, UserNum, UserName, TransactionType, TransactionAmount, TransactionPrice, TransactionCost, RefferenceCode, ProofImage)
                 VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9);
                 ",
                vec![
                    &transaction_id as &dyn tiberius::ToSql,
                    &transaction.user_num,
                    &transaction.user_name,
                    &transaction.r#type,
                    &transaction.points,
                    &transaction.amount,
                    &transaction.amount,
                    &gcash_info.reference_number,
                    &maybe_proof
                ],
            )
        } else {
            (
                "
                 INSERT INTO [dbo].[WebTransaction]
                 (TransactionID,UserNum, UserName, TransactionType, TransactionAmount, TransactionPrice, TransactionCost, RefferenceCode)
                 VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8);
                 ",
                vec![
                    &transaction_id as &dyn tiberius::ToSql,
                    &transaction.user_num,
                    &transaction.user_name,
                    &transaction.r#type,
                    &transaction.points,
                    &transaction.amount,
                    &transaction.amount,
                    &gcash_info.reference_number,
                ],
            )
        };
        self.execute_query(query_str, params.as_slice()).await?;
        Ok(transaction_id)
    }

    async fn transact(&self, transaction: Transaction) -> Result<Uuid> {
        let transaction_uuid = Uuid::new_v4();
        let query = "INSERT INTO [dbo].[WebTransaction]
            (TransactionID, UserNum, UserName, TransactionType, TransactionAmount, TransactionPrice, TransactionCost)
            VALUES (@P1, @P2, @P3, @P4, @P5, @P6, @P7);";

        let params: Vec<&dyn tiberius::ToSql> = vec![
            &transaction_uuid,
            &transaction.user_num,
            &transaction.user_name,
            &transaction.r#type,
            &transaction.points,
            &transaction.amount,
            &transaction.amount,
        ];

        self.execute_query(query, params.as_slice()).await?;
        Ok(transaction_uuid)
    }
}
