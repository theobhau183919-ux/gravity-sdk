// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

#[allow(unused_imports)]
use anyhow::{format_err, Result};
use axum::response::IntoResponse;
#[cfg(feature = "failpoints")]
use axum::Json;
#[cfg(feature = "failpoints")]
use gaptos::aptos_logger::prelude::*;
use serde::{Deserialize, Serialize};

const FAILPOINT_AUTH_TOKEN_ENV: &str = "FAILPOINT_AUTH_TOKEN";

#[derive(Deserialize, Serialize)]
pub struct FailpointConf {
    name: String,
    actions: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FailpointConfResponse {
    pub response: String,
    // tx status
}

#[cfg(feature = "failpoints")]
pub async fn set_failpoint(
    request: FailpointConf,
    auth_token: Option<String>,
) -> impl IntoResponse {
    let expected = std::env::var(FAILPOINT_AUTH_TOKEN_ENV).ok().filter(|token| !token.is_empty());
    if expected.is_none() || auth_token != expected {
        return (
            axum::http::StatusCode::FORBIDDEN,
            "Failpoint endpoint is disabled or unauthorized".to_string(),
        )
            .into_response();
    }
    match fail::cfg(&request.name, &request.actions) {
        Ok(_) => {
            info!("Configured failpoint {} to {}", request.name, request.actions);
            let response = format!("Set failpoint {}", request.name);
            Json(FailpointConfResponse { response }).into_response()
        }
        Err(e) => {
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to set failpoint: {e}"))
                .into_response()
        }
    }
}

#[cfg(not(feature = "failpoints"))]
pub async fn set_failpoint(_: FailpointConf, _: Option<String>) -> impl IntoResponse {
    (
        axum::http::StatusCode::BAD_REQUEST,
        "Failpoints are not enabled at a feature level".to_string(),
    )
        .into_response()
}
