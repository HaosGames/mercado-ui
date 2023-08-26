use std::str::FromStr;

use crate::URL;
use leptos::{SignalSet, WriteSignal};
use mercado::api::*;
use mercado::client::Client;
use mercado::secp256k1::ecdsa::Signature;

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
pub async fn get_prediction_bets(request: PredictionRequest) -> Result<Vec<Bet>, String> {
    let client = Client::new(URL.to_string());
    client
        .get_prediction_bets(request)
        .await
        .map_err(map_any_err)
}

pub async fn accept_nomination(
    request: AcceptNominationRequest,
    access: AccessRequest,
) -> Result<(), String> {
    let client = Client::new(URL.to_string());
    client
        .accept_nomination(request, access)
        .await
        .map_err(map_any_err)
}
pub async fn get_login_challenge(user: String) -> Result<String, String> {
    let client = Client::new(URL.to_string());
    let user = UserPubKey::from_str(user.as_str())
        .map_err(|e| e.into())
        .map_err(map_any_err)?;
    client.get_login_challenge(user).await.map_err(map_any_err)
}
pub async fn try_login(
    (user, signature, set_access): (String, String, WriteSignal<Option<AccessRequest>>),
) -> Result<String, String> {
    let request = LoginRequest {
        user: UserPubKey::from_str(user.as_str())
            .map_err(|e| e.into())
            .map_err(map_any_err)?,
        sig: Signature::from_str(signature.as_str())
            .map_err(|e| e.into())
            .map_err(map_any_err)?,
    };
    let client = Client::new(URL.to_string());
    client
        .try_login(request.clone())
        .await
        .map_err(map_any_err)?;
    set_access.set(Some(AccessRequest {
        user: request.user,
        sig: request.sig,
    }));
    Ok(format!("Successfull login as {}", user))
}
pub async fn check_login(access: AccessRequest) -> Result<String, String> {
    let client = Client::new(URL.to_string());
    client.check_login(access).await.map_err(map_any_err)?;
    Ok("".to_string())
}
