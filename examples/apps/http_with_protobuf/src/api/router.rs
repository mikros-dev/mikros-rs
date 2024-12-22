// Code generated by protoc-gen-mikros-extensions. DO NOT EDIT.

use axum::extract::Path;
use axum::extract::{Extension, State};
use axum::http::header::HeaderMap;
use axum::routing::{delete, get, post, put};
use axum::Json;
use mikros::axum;
use mikros::serde_derive;
use mikros::{errors as merrors, http::ServiceState};
use serde_derive::{Deserialize, Serialize};
use std::sync::Arc;
use tonic::Request;

use crate::card;

#[derive(Serialize)]
struct CardWire {
    pub id: String,
    pub owner_name: String,
    pub card_id: String,
    pub created_at: Option<prost_wkt_types::Timestamp>,
    pub updated_at: Option<prost_wkt_types::Timestamp>,
}

impl From<card::CardWire> for CardWire {
    fn from(m: card::CardWire) -> Self {
        Self {
            id: m.id,
            owner_name: m.owner_name,
            card_id: m.card_id,
            created_at: m.created_at,
            updated_at: m.updated_at,
        }
    }
}

#[derive(Deserialize)]
struct CreateCardRequest {
    pub owner_name: String,
    pub card_id: String,
    pub debug: bool,
}

impl From<CreateCardRequest> for card::CreateCardRequest {
    fn from(m: CreateCardRequest) -> Self {
        Self {
            owner_name: m.owner_name,
            card_id: m.card_id,
            debug: m.debug,
        }
    }
}

#[derive(Deserialize)]
struct DeleteCardRequest {
    pub id: String,
    pub debug: bool,
}

impl From<DeleteCardRequest> for card::DeleteCardRequest {
    fn from(m: DeleteCardRequest) -> Self {
        Self {
            id: m.id,
            debug: m.debug,
        }
    }
}

#[derive(Deserialize)]
struct GetCardRequest {
    pub id: String,
    pub debug: bool,
}

impl From<GetCardRequest> for card::GetCardRequest {
    fn from(m: GetCardRequest) -> Self {
        Self {
            id: m.id,
            debug: m.debug,
        }
    }
}

#[derive(Deserialize)]
struct UpdateCardRequest {
    pub id: String,
    pub owner_name: String,
    pub card_id: String,
    pub debug: bool,
}

impl From<UpdateCardRequest> for card::UpdateCardRequest {
    fn from(m: UpdateCardRequest) -> Self {
        Self {
            id: m.id,
            owner_name: m.owner_name,
            card_id: m.card_id,
            debug: m.debug,
        }
    }
}

#[derive(Serialize)]
struct CreateCardResponse {
    pub card: CardWire,
}

impl From<card::CreateCardResponse> for CreateCardResponse {
    fn from(m: card::CreateCardResponse) -> Self {
        Self {
            card: m.card.unwrap().into(),
        }
    }
}

#[derive(Serialize)]
struct DeleteCardResponse {
    pub card: CardWire,
}

impl From<card::DeleteCardResponse> for DeleteCardResponse {
    fn from(m: card::DeleteCardResponse) -> Self {
        Self {
            card: m.card.unwrap().into(),
        }
    }
}

#[derive(Serialize)]
struct GetCardResponse {
    pub card: CardWire,
}

impl From<card::GetCardResponse> for GetCardResponse {
    fn from(m: card::GetCardResponse) -> Self {
        Self {
            card: m.card.unwrap().into(),
        }
    }
}

#[derive(Serialize)]
struct UpdateCardResponse {
    pub card: CardWire,
}

impl From<card::UpdateCardResponse> for UpdateCardResponse {
    fn from(m: card::UpdateCardResponse) -> Self {
        Self {
            card: m.card.unwrap().into(),
        }
    }
}

#[derive(Deserialize)]
struct CreateCardRequestBody {
    pub owner_name: String,
    pub card_id: String,
}

async fn create_card(
    State(state): State<Arc<mikros::Mutex<ServiceState>>>,
    Extension(router): Extension<Router>,
    headers: HeaderMap,
    Json(body): Json<CreateCardRequestBody>,
) -> Result<Json<CreateCardResponse>, merrors::ServiceError> {
    let context = state.lock().await.context();

    // Retrieve all arguments to create the endpoint main structure.
    let args = CreateCardRequest {
        debug: mikros::http::header::to_bool(context.clone(), &headers, "debug")?,
        owner_name: body.owner_name,
        card_id: body.card_id,
    };

    let input: card::CreateCardRequest = args.into();
    let mut request = Request::new(input);

    // Adds mikros context inside the request for the wrapper to also access it.
    request.extensions_mut().insert(context.clone());

    match router.wrapper.create_card(request).await {
        Ok(response) => Ok(Json(response.into_inner().into())),
        Err(error) => {
            // Translates the wrapper error into the endpoint response.
            let e: merrors::ServiceError = error.into();
            Err(e.into())
        }
    }
}

async fn get_card(
    State(state): State<Arc<mikros::Mutex<ServiceState>>>,
    Extension(router): Extension<Router>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<GetCardResponse>, merrors::ServiceError> {
    let context = state.lock().await.context();

    // Retrieve all arguments to create the endpoint main structure.
    let args = GetCardRequest {
        id: id,
        debug: mikros::http::header::to_bool(context.clone(), &headers, "debug")?,
    };

    let input: card::GetCardRequest = args.into();
    let mut request = Request::new(input);

    // Adds mikros context inside the request for the wrapper to also access it.
    request.extensions_mut().insert(context.clone());

    // Translates the wrapper response into the endpoint response.
    let res = router.wrapper.get_card(request).await?;
    Ok(Json(res.into_inner().into()))
}

#[derive(Deserialize)]
struct UpdateCardRequestBody {
    pub owner_name: String,
    pub card_id: String,
}

async fn update_card(
    State(state): State<Arc<mikros::Mutex<ServiceState>>>,
    Extension(router): Extension<Router>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Json(body): Json<UpdateCardRequestBody>,
) -> Result<Json<UpdateCardResponse>, merrors::ServiceError> {
    let context = state.lock().await.context();

    // Retrieve all arguments to create the endpoint main structure.
    let args = UpdateCardRequest {
        id: id,
        debug: mikros::http::header::to_bool(context.clone(), &headers, "debug")?,
        owner_name: body.owner_name,
        card_id: body.card_id,
    };

    let input: card::UpdateCardRequest = args.into();
    let mut request = Request::new(input);

    // Adds mikros context inside the request for the wrapper to also access it.
    request.extensions_mut().insert(context.clone());

    // Translates the wrapper response into the endpoint response.
    let res = router.wrapper.update_card(request).await?;
    Ok(Json(res.into_inner().into()))
}

async fn delete_card(
    State(state): State<Arc<mikros::Mutex<ServiceState>>>,
    Extension(router): Extension<Router>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<DeleteCardResponse>, merrors::ServiceError> {
    let context = state.lock().await.context();

    // Retrieve all arguments to create the endpoint main structure.
    let args = DeleteCardRequest {
        id: id,
        debug: mikros::http::header::to_bool(context.clone(), &headers, "debug")?,
    };

    let input: card::DeleteCardRequest = args.into();
    let mut request = Request::new(input);

    // Adds mikros context inside the request for the wrapper to also access it.
    request.extensions_mut().insert(context.clone());

    // Translates the wrapper response into the endpoint response.
    let res = router.wrapper.delete_card(request).await?;
    Ok(Json(res.into_inner().into()))
}

#[derive(Clone)]
pub struct Router {
    wrapper: Arc<dyn card::card_service_server::CardService>,
}

impl Router {
    pub fn new(server: Arc<dyn card::card_service_server::CardService>) -> Self {
        Self { wrapper: server }
    }

    pub fn routes(self) -> axum::Router<Arc<mikros::Mutex<ServiceState>>> {
        axum::Router::new()
            .route("/card/v1/cards", post(create_card))
            .route("/card/v1/cards/:id", get(get_card))
            .route("/card/v1/cards/:id", put(update_card))
            .route("/card/v1/cards/:id", delete(delete_card))
            .layer(Extension(self.clone()))
    }
}
