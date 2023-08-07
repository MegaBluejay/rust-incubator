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
use juniper::{http::GraphQLBatchRequest, EmptySubscription};
use sea_orm::DatabaseConnection;
use thiserror::Error;

use crate::{
    api::{Context, Mutation, Root, User},
    db::{self, get_user, Claims, SeaDb},
};

struct TheState {
    db: DatabaseConnection,
    schema: Root,
    secret: Vec<u8>,
    graphql_url: String,
}

pub async fn server(
    db: DatabaseConnection,
    addr: &SocketAddr,
    secret: &[u8],
) -> Result<(), hyper::Error> {
    let app = Router::new()
        .route("/", get(graphiql_handler))
        .route("/graphql", post(graphql_handler))
        .with_state(Arc::new(TheState {
            db,
            schema: Root::new(crate::api::Query, Mutation, EmptySubscription::new()),
            secret: secret.to_owned(),
            graphql_url: "/graphql".to_owned(),
        }));

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
    State(state): State<Arc<TheState>>,
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
        None => Err(AuthError::NoToken),
    }
    .map_err(|err| err.to_string());

    let ctx = Context::new(
        SeaDb::new(state.db.clone(), EncodingKey::from_secret(&state.secret)),
        current_user,
        5,
    );

    let response = request.execute(&state.schema, &ctx).await;
    let status = if response.is_ok() {
        StatusCode::OK
    } else {
        StatusCode::BAD_REQUEST
    };
    let ser_response = serde_json::to_string(&response).unwrap();
    (status, ser_response)
}

async fn graphiql_handler(State(state): State<Arc<TheState>>) -> Html<String> {
    let html = include_str!("../graphiql.html");
    Html(html.replace("{{GRAPHQL_URL}}", &state.graphql_url))
}

#[derive(Debug, Error)]
enum AuthError {
    #[error(transparent)]
    Db(#[from] db::Error),
    #[error(transparent)]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("no auth token given")]
    NoToken,
}

async fn authenticate(
    token: &str,
    key: &DecodingKey,
    db: &DatabaseConnection,
) -> Result<User, AuthError> {
    let id = jsonwebtoken::decode::<Claims>(token, key, &Validation::new(Algorithm::HS512))?
        .claims
        .id;
    get_user(db, id).await.map_err(Into::into)
}
