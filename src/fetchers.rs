use std::str::FromStr;

use crate::{MercadoState, URL};
use leptos::{ReadSignal, Resource, RwSignal, SignalGetUntracked, SignalSet, WriteSignal};
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
pub async fn get_judges(
    prediction: Option<RowId>,
    user: Option<UserPubKey>,
) -> Result<Vec<JudgePublic>, String> {
    let request = PredictionUserRequest { user, prediction };
    client().get_judges(request).await.map_err(map_any_err)
}
pub async fn get_judge(
    prediction: RowId,
    user: UserPubKey,
    access: ReadSignal<MercadoState>,
) -> Result<Judge, String> {
    if let Some(access) = access.get_untracked().access {
        let request = JudgeRequest { prediction, user };
        client()
            .get_judge(request, access)
            .await
            .map_err(map_any_err)
    } else {
        Err("Not logged in".to_string())
    }
}
pub async fn get_bets(
    request: PredictionUserRequest,
    access: AccessRequest,
) -> Result<Vec<Bet>, String> {
    client()
        .get_bets(request, access)
        .await
        .map_err(map_any_err)
}
pub async fn get_prediction_ratio(request: PredictionRequest) -> Result<(Sats, Sats), String> {
    client()
        .get_prediction_ratio(request)
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
    (user, signature, challenge, set_state): (String, String, String, WriteSignal<MercadoState>),
) -> Result<String, String> {
    let user = UserPubKey::from_str(user.as_str())
        .map_err(|e| e.into())
        .map_err(map_any_err)?;
    let sig = Signature::from_str(signature.as_str())
        .map_err(|e| e.into())
        .map_err(map_any_err)?;
    let request = LoginRequest {
        user,
        sig,
        challenge,
    };
    client()
        .try_login(request.clone())
        .await
        .map_err(map_any_err)?;
    let access = AccessRequest {
        user: request.user,
        sig: request.sig,
        challenge: request.challenge,
    };
    let user_detail = get_user(user, access.clone()).await?;
    set_state.set(MercadoState {
        access: Some(access.clone()),
        user: Some(user_detail),
    });
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
pub async fn get_user(user: UserPubKey, access: AccessRequest) -> Result<UserResponse, String> {
    let user = client().get_user(user, access).await.map_err(map_any_err)?;
    Ok(user)
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
pub async fn new_prediction(request: NewPredictionRequest) -> Result<RowId, String> {
    client().new_prediction(request).await.map_err(map_any_err)
}
pub async fn add_bet(request: AddBetRequest, access: AccessRequest) -> Result<Payment, String> {
    client().add_bet(request, access).await.map_err(map_any_err)
}
pub async fn make_decision(
    request: MakeDecisionRequest,
    access: AccessRequest,
) -> Result<(), String> {
    client()
        .make_decision(request, access)
        .await
        .map_err(map_any_err)
}
pub async fn get_balance(user: UserPubKey, access: AccessRequest) -> Result<Sats, String> {
    client()
        .get_balance(user, access)
        .await
        .map_err(map_any_err)
}
pub async fn get_available_balance(
    user: UserPubKey,
    access: AccessRequest,
) -> Result<Sats, String> {
    client()
        .get_available_balance(user, access)
        .await
        .map_err(map_any_err)
}
pub async fn get_balances_for(access: Option<AccessRequest>) -> Result<(Sats, Sats), String> {
    if let Some(access) = access {
        let balance = get_balance(access.clone().user, access.clone()).await?;
        let available_balance = get_available_balance(access.clone().user, access).await?;
        Ok((available_balance, balance))
    } else {
        Err("Not logged in".to_string())
    }
}
pub async fn force_decision_period(prediction: RowId, access: AccessRequest) -> Result<(), String> {
    client()
        .force_decision_period(prediction, access)
        .await
        .map_err(map_any_err)
}
pub async fn fetch_rw_signal<T>(signal: RwSignal<T>) -> T
where
    T: Clone,
{
    signal.get_untracked()
}
