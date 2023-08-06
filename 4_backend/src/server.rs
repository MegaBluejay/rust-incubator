use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::State,
    headers::{authorization::Bearer, Authorization},
    response::Html,
    routing::{get, post},
    Json, Router, TypedHeader,
};
use hyper::StatusCode;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use juniper::{
    http::{graphiql::graphiql_source, GraphQLBatchRequest},
    EmptySubscription,
};
use sea_orm::DatabaseConnection;

use crate::{
    api::{Context, Mutation, Root, User, WrappedDatabase},
    db::{get_user, Claims, SeaDb},
};

#[derive(Clone)]
struct TheState {
    db: DatabaseConnection,
    schema: Arc<Root>,
    secret: Vec<u8>,
}

pub async fn server(
    db: DatabaseConnection,
    addr: &SocketAddr,
    secret: &[u8],
) -> Result<(), hyper::Error> {
    let app = Router::new()
        .route("/", get(graphiql_handler))
        .route("/graphql", post(graphql_handler))
        .with_state(TheState {
            db,
            schema: Arc::new(Root::new(
                crate::api::Query,
                Mutation,
                EmptySubscription::new(),
            )),
            secret: secret.to_owned(),
        });

    axum::Server::bind(addr)
        .serve(app.into_make_service())
        .await
}

struct MaybeHeader<T>(Option<T>);

impl<T: axum::headers::Header> axum::headers::Header for MaybeHeader<T> {
    fn name() -> &'static axum::http::HeaderName {
        T::name()
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i axum::http::HeaderValue>,
    {
        Ok(Self(T::decode(values).ok()))
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, values: &mut E) {
        if let Some(t) = &self.0 {
            t.encode(values);
        }
    }
}

async fn graphql_handler(
    State(state): State<TheState>,
    TypedHeader(MaybeHeader(auth)): TypedHeader<MaybeHeader<Authorization<Bearer>>>,
    request: Json<GraphQLBatchRequest>,
) -> (StatusCode, String) {
    let current_user = match auth {
        Some(Authorization(bearer)) => {
            authenticate(
                bearer.token(),
                &DecodingKey::from_secret(&state.secret),
                &state.db,
            )
            .await
        }
        None => None,
    };

    let ctx = Context {
        current_user,
        db: Box::new(WrappedDatabase::new(SeaDb {
            db: state.db,
            key: EncodingKey::from_secret(&state.secret),
        })),
    };

    let response = request.execute(state.schema.as_ref(), &ctx).await;
    let status = if response.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    let ser_response = serde_json::to_string(&response).unwrap();
    (status, ser_response)
}

async fn graphiql_handler() -> Html<String> {
    Html(graphiql_source("/graphql", None))
}

async fn authenticate(token: &str, key: &DecodingKey, db: &DatabaseConnection) -> Option<User> {
    let id = jsonwebtoken::decode::<Claims>(token, key, &Validation::new(Algorithm::HS512))
        .ok()?
        .claims
        .id;
    get_user(db, id).await.ok().flatten()
}
