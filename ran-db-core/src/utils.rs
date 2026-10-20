use anyhow::Result;
use ran_models::BaseDataCountResult;
use tiberius::Row;

pub fn list_response(results: Vec<Vec<Row>>) -> Result<BaseDataCountResult<Row>> {
    let result_sets = results.into_iter().next().unwrap_or_default();

    let total_count = result_sets
        .first()
        .and_then(|row| row.get::<i32, _>("TotalRecords"))
        .unwrap_or(0);

    Ok(BaseDataCountResult {
        data: result_sets,
        total_records: total_count
    })
}
