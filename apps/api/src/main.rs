use std::{convert::Infallible, time::Duration};

use axum::{
    extract::Query,
    http::{header, HeaderValue, Method, StatusCode},
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Json, Router,
};
use futures_util::{stream, Stream};
use matchlens_match_engine::{demo_events, snapshot_at};
use matchlens_shared::{MatchEvent, MatchSnapshot};
use serde::{Deserialize, Serialize};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing_subscriber::EnvFilter;

const DEMO_MATCH_ID: &str = "demo-match-001";
const DEMO_DURATION_SECONDS: u32 = 420;

#[derive(Serialize)]
struct Health {
    status: &'static str,
    service: &'static str,
}

async fn health() -> Json<Health> {
    Json(Health {
        status: "ok",
        service: "matchlens-api",
    })
}

async fn events() -> Json<Vec<MatchEvent>> {
    Json(demo_events())
}

#[derive(Deserialize)]
struct SnapshotQuery {
    at: Option<u32>,
}

async fn snapshot(
    Query(query): Query<SnapshotQuery>,
) -> Result<Json<MatchSnapshot>, (StatusCode, &'static str)> {
    let at = query
        .at
        .unwrap_or(DEMO_DURATION_SECONDS)
        .min(DEMO_DURATION_SECONDS);
    snapshot_at(DEMO_MATCH_ID, &demo_events(), at)
        .map(Json)
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "demo data validation failed",
            )
        })
}

async fn event_stream() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let events = demo_events();
    let stream = stream::unfold((0usize, events), |(index, events)| async move {
        if index > events.len() {
            return None;
        }
        if index > 0 {
            tokio::time::sleep(Duration::from_millis(1_250)).await;
        }
        let payload = if index == events.len() {
            Event::default().event("replay-complete").data("complete")
        } else {
            Event::default()
                .event("match-event")
                .json_data(&events[index])
                .expect("static demo event is serializable")
        };
        Some((Ok(payload), (index + 1, events)))
    });
    Sse::new(stream).keep_alive(KeepAlive::default())
}

fn app() -> Router {
    let origin = std::env::var("WEB_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".into());
    let origin: HeaderValue = origin
        .parse()
        .expect("WEB_ORIGIN must be a valid header value");
    let cors = CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET])
        .allow_headers([header::CONTENT_TYPE]);

    Router::new()
        .route("/api/v1/health", get(health))
        .route("/api/v1/matches/demo/events", get(events))
        .route("/api/v1/matches/demo/snapshot", get(snapshot))
        .route("/api/v1/matches/demo/stream", get(event_stream))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", port))
        .await
        .expect("bind API port");
    tracing::info!(port, "MatchLens API listening");
    axum::serve(listener, app()).await.expect("serve HTTP");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_is_200() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn snapshot_contains_one_goal_at_420_seconds() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/matches/demo/snapshot?at=420")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["score"]["home"], 1);
        assert_eq!(json["events_processed"], 9);
    }
}
