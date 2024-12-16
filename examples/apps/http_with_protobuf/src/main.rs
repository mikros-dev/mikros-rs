mod api;
mod card {
    include!("generated/card.rs");
}

use std::sync::Arc;
use std::collections::HashMap;

use mikros::service::{builder::ServiceBuilder, context};
use tonic::{Request, Response, Status};

use api::router::Router;

#[derive(Clone, Default)]
pub struct Service {
    cards: Arc<mikros::Mutex<HashMap<String, card::CardWire>>>
}

#[tonic::async_trait]
impl card::card_service_server::CardService for Service {
    async fn create_card(&self, request: Request<card::CreateCardRequest>) -> Result<Response<card::CreateCardResponse>, Status> {
        let ctx = context::from_request(&request);
        ctx.logger().info("create_card RPC called");

        let request = request.into_inner();
        let card = card::CardWire {
            id: uuid::Uuid::new_v4().to_string(),
            owner_name: request.owner_name,
            card_id: request.card_id,
            created_at: Some(chrono::Utc::now().into()),
            updated_at: None,
        };

        self.cards.lock().await.insert(card.id.clone(), card.clone());

        Ok(Response::new(card::CreateCardResponse{
            card: Some(card),
        }))
    }

    async fn get_card(&self, request: Request<card::GetCardRequest>) -> Result<Response<card::GetCardResponse>, Status> {
        let ctx = context::from_request(&request);
        ctx.logger().info("get_card RPC called");

        let request = request.into_inner();
        match self.cards.lock().await.get(&request.id) {
            None => Err(Status::not_found("card not found")),
            Some(card) => {
                Ok(Response::new(card::GetCardResponse{
                    card: Some(card.clone()),
                }))
            }
        }
    }

    async fn update_card(&self, request: Request<card::UpdateCardRequest>) -> Result<Response<card::UpdateCardResponse>, Status> {
        let ctx = context::from_request(&request);
        ctx.logger().info("update_card RPC called");

        let request = request.into_inner();

        match self.cards.lock().await.get_mut(&request.id) {
            None => {
                Err(Status::not_found("not found"))
            }
            Some(card) => {
                card.card_id = request.card_id;
                card.owner_name = request.owner_name;
                card.updated_at = Some(chrono::Utc::now().into());

                Ok(Response::new(card::UpdateCardResponse{
                    card: Some(card.clone()),
                }))
            }
        }
    }

    async fn delete_card(&self, request: Request<card::DeleteCardRequest>) -> Result<Response<card::DeleteCardResponse>, Status> {
        let ctx = context::from_request(&request);
        ctx.logger().info("delete_card RPC called");

        let request = request.into_inner();
        let card = self.cards.lock().await.remove(&request.id);
        if let None = card {
            return Err(Status::not_found("not found"))
        }

        Ok(Response::new(card::DeleteCardResponse{
            card: Some(card.unwrap()),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let greeter = Arc::new(Service::default());
    let routes = Router::new(greeter).routes();

    let mut svc = ServiceBuilder::default()
        .http(routes)
        .build()?;

    Ok(svc.start().await?)
}
