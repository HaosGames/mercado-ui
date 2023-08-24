use crate::URL;
use mercado::api::*;
use mercado::client::Client;

pub async fn get_predictions(_: ()) -> Result<Vec<PredictionOverviewResponse>, String> {
    let client = Client::new(URL.to_string());
    client.get_predictions().await.map_err(map_any_err)
}
pub async fn get_prediction_overview(
    prediction: RowId,
) -> Result<PredictionOverviewResponse, String> {
    let client = Client::new(URL.to_string());
    let request = PredictionRequest {
        user: None,
        prediction,
    };
    client
        .get_prediction_overview(request)
        .await
        .map_err(map_any_err)
}
pub async fn get_prediction_judges(prediction: RowId) -> Result<Vec<Judge>, String> {
    let client = Client::new(URL.to_string());
    let request = PredictionRequest {
        user: None,
        prediction,
    };
    client
        .get_prediction_judges(request)
        .await
        .map_err(map_any_err)
}
