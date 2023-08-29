use std::str::FromStr;

use crate::URL;
use leptos::{SignalSet, WriteSignal};
use mercado::api::*;
use mercado::client::Client;
use mercado::secp256k1::ecdsa::Signature;

pub fn client() -> Client {
    Client::new(URL.to_string())
}

pub async fn get_predictions(_: ()) -> Result<Vec<PredictionOverviewResponse>, String> {
    client().get_predictions().await.map_err(map_any_err)
}
pub async fn get_prediction_overview(
    prediction: RowId,
) -> Result<PredictionOverviewResponse, String> {
    let request = PredictionRequest {
        user: None,
        prediction,
    };
    client()
        .get_prediction_overview(request)
        .await
        .map_err(map_any_err)
}
pub async fn get_prediction_judges(prediction: RowId) -> Result<Vec<Judge>, String> {
    let request = PredictionRequest {
        user: None,
        prediction,
    };
    client()
        .get_prediction_judges(request)
        .await
        .map_err(map_any_err)
}
pub async fn get_prediction_bets(request: PredictionRequest) -> Result<Vec<Bet>, String> {
    client()
        .get_prediction_bets(request)
        .await
        .map_err(map_any_err)
}

pub async fn accept_nomination(
    request: NominationRequest,
    access: AccessRequest,
) -> Result<(), String> {
    client()
        .accept_nomination(request, access)
        .await
        .map_err(map_any_err)
}
pub async fn refuse_nomination(
    request: NominationRequest,
    access: AccessRequest,
) -> Result<(), String> {
    client()
        .refuse_nomination(request, access)
        .await
        .map_err(map_any_err)
}
pub async fn create_login_challenge(user: String) -> Result<String, String> {
    let user = UserPubKey::from_str(user.as_str())
        .map_err(|e| e.into())
        .map_err(map_any_err)?;
    client()
        .create_login_challenge(user)
        .await
        .map_err(map_any_err)
}
pub async fn try_login(
    (user, signature, challenge, set_access): (
        String,
        String,
        String,
        WriteSignal<Option<AccessRequest>>,
    ),
) -> Result<String, String> {
    let request = LoginRequest {
        user: UserPubKey::from_str(user.as_str())
            .map_err(|e| e.into())
            .map_err(map_any_err)?,
        sig: Signature::from_str(signature.as_str())
            .map_err(|e| e.into())
            .map_err(map_any_err)?,
        challenge,
    };
    client()
        .try_login(request.clone())
        .await
        .map_err(map_any_err)?;
    set_access.set(Some(AccessRequest {
        user: request.user,
        sig: request.sig,
        challenge: request.challenge,
    }));
    Ok(format!("Successfull login as {}", user))
}
pub async fn check_login(access: Option<AccessRequest>) -> Result<String, String> {
    if let Some(access) = access {
        client().check_login(access).await.map_err(map_any_err)?;
    } else {
        return Err("Not logged in".to_string());
    }
    Ok("".to_string())
}
pub async fn get_username(user: UserPubKey) -> Result<String, String> {
    let name = client().get_username(user).await.map_err(map_any_err)?;
    Ok(name)
}
pub async fn my_bets(access: Option<AccessRequest>) -> Result<Vec<Bet>, String> {
    if let Some(access) = access {
        client()
            .get_bets(
                PredictionUserRequest {
                    prediction: None,
                    user: Some(access.user),
                },
                access,
            )
            .await
            .map_err(map_any_err)
    } else {
        Err("You need to login to see your bets".to_string())
    }
}
