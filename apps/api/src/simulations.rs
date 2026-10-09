use axum::{
    extract::{rejection::JsonRejection, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use reactcoach_decision_engine::simulation::{simulate, SimulationReport, SimulationRequest};
use serde::Serialize;
use std::{sync::Arc, time::Duration};
use tokio::sync::Semaphore;

#[derive(Serialize)]
struct ErrorBody {
    error: ErrorDetail,
}
#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: &'static str,
    field: Option<&'static str>,
}
type ApiError = (StatusCode, Json<ErrorBody>);
fn error(
    status: StatusCode,
    code: &'static str,
    message: &'static str,
    field: Option<&'static str>,
) -> ApiError {
    (
        status,
        Json(ErrorBody {
            error: ErrorDetail {
                code,
                message,
                field,
            },
        }),
    )
}

pub(super) fn router() -> Router {
    router_with_limit(Arc::new(Semaphore::new(4)))
}
fn router_with_limit(limit: Arc<Semaphore>) -> Router {
    Router::new()
        .route("/api/v1/simulations/open-play", post(open_play))
        .with_state(limit)
}

async fn open_play(
    State(limit): State<Arc<Semaphore>>,
    input: Result<Json<SimulationRequest>, JsonRejection>,
) -> Result<Json<SimulationReport>, ApiError> {
    let Json(request) = input.map_err(|rejection| {
        error(
            rejection.status(),
            "invalid_json",
            "invalid or oversized simulation JSON; check the versioned contract and content type",
            None,
        )
    })?;
    let permit = limit.try_acquire_owned().map_err(|_| {
        error(
            StatusCode::SERVICE_UNAVAILABLE,
            "simulation_busy",
            "simulation capacity reached; retry later",
            None,
        )
    })?;
    // Permit lives inside the worker: timeout cannot release capacity while work still runs.
    let worker = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        simulate(request)
    });
    let result = tokio::time::timeout(Duration::from_secs(2), worker)
        .await
        .map_err(|_| {
            error(
                StatusCode::GATEWAY_TIMEOUT,
                "simulation_timeout",
                "bounded simulation response timed out",
                None,
            )
        })?
        .map_err(|_| {
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "simulation_failed",
                "simulation worker failed",
                None,
            )
        })?;
    result.map(Json).map_err(|e| {
        error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "invalid_scenario",
            e.message,
            Some(e.field),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        extract::DefaultBodyLimit,
        http::Request,
    };
    use reactcoach_decision_engine::simulation::{demo_request, SimulationReport};
    use tower::ServiceExt;

    fn request(body: impl Into<Body>) -> Request<Body> {
        Request::builder()
            .method("POST")
            .uri("/api/v1/simulations/open-play")
            .header("content-type", "application/json")
            .body(body.into())
            .unwrap()
    }
    async fn json_response(app: Router, payload: String, status: StatusCode) -> serde_json::Value {
        let response = app.oneshot(request(payload)).await.unwrap();
        assert_eq!(response.status(), status);
        serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap()).unwrap()
    }
    #[tokio::test]
    async fn simulation_contract_roundtrips_and_is_deterministic() {
        let payload = serde_json::to_string(&demo_request()).unwrap();
        let first = json_response(crate::app(), payload.clone(), StatusCode::OK).await;
        let second = json_response(crate::app(), payload, StatusCode::OK).await;
        assert_eq!(first, second);
        let report: SimulationReport = serde_json::from_value(first).unwrap();
        assert_eq!(report.baseline.steps.len(), 41);
        assert_eq!(report.input.scenario_version, "open_play_spatial_v1");
    }
    #[tokio::test]
    async fn malformed_unknown_nonfinite_and_wrong_types_are_structured() {
        let valid = serde_json::to_value(demo_request()).unwrap();
        let mut unknown = valid.clone();
        unknown["extra"] = true.into();
        let mut float = valid.clone();
        float["initial_state"]["assessed"]["position_mm"]["x"] = 1.5.into();
        for payload in [
            "{".to_owned(),
            "{}".into(),
            "{\"x\":NaN}".into(),
            unknown.to_string(),
            float.to_string(),
        ] {
            let response = crate::app().oneshot(request(payload)).await.unwrap();
            assert!(response.status().is_client_error());
            let json: serde_json::Value =
                serde_json::from_slice(&to_bytes(response.into_body(), 8192).await.unwrap())
                    .unwrap();
            assert_eq!(json["error"]["code"], "invalid_json");
        }
    }
    #[tokio::test]
    async fn semantic_errors_are_structured_with_field() {
        let mut scenario = demo_request();
        scenario.config.duration_ms = u32::MAX;
        let json = json_response(
            crate::app(),
            serde_json::to_string(&scenario).unwrap(),
            StatusCode::UNPROCESSABLE_ENTITY,
        )
        .await;
        assert_eq!(json["error"]["code"], "invalid_scenario");
        assert_eq!(json["error"]["field"], "config");
    }
    #[tokio::test]
    async fn oversized_body_and_missing_content_type_are_rejected() {
        let json = json_response(
            crate::app(),
            "x".repeat(17 * 1024),
            StatusCode::PAYLOAD_TOO_LARGE,
        )
        .await;
        assert_eq!(json["error"]["code"], "invalid_json");
        let response = crate::app()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/simulations/open-play")
                    .body(Body::from("{}"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNSUPPORTED_MEDIA_TYPE);
    }
    #[tokio::test]
    async fn concurrency_limit_returns_structured_busy() {
        let limit = Arc::new(Semaphore::new(1));
        let _held = limit.clone().acquire_owned().await.unwrap();
        let app = router_with_limit(limit).layer(DefaultBodyLimit::max(16 * 1024));
        let json = json_response(
            app,
            serde_json::to_string(&demo_request()).unwrap(),
            StatusCode::SERVICE_UNAVAILABLE,
        )
        .await;
        assert_eq!(json["error"]["code"], "simulation_busy");
    }
    #[tokio::test]
    async fn maximum_rollout_payload_is_bounded() {
        let mut scenario = demo_request();
        scenario.config.duration_ms = 6000;
        let response = crate::app()
            .oneshot(request(serde_json::to_string(&scenario).unwrap()))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        let report: SimulationReport = serde_json::from_slice(&bytes).unwrap();
        assert!(report.baseline.steps.len() <= 61);
        assert!(report.alternatives.len() <= 5);
        assert!(report
            .alternatives
            .iter()
            .filter_map(|a| a.trace.as_ref())
            .all(|t| t.steps.len() <= 61));
    }
}
