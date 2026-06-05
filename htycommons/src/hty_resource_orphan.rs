use crate::common::HtyResponse;
use crate::models::ClaimHtyResourcesReq;
use crate::web::get_uc_url;
use tracing::debug;

/// 业务保存成功后认领 UC 资源，将 `is_orphan` 置为 false。
pub async fn uc_claim_hty_resources(
    hty_resource_ids: &[String],
    _claimer_hty_id: &str,
    sudoer: &str,
    host: &str,
) -> anyhow::Result<usize> {
    let ids: Vec<String> = hty_resource_ids
        .iter()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    if ids.is_empty() {
        return Ok(0);
    }
    let client = reqwest::Client::new();
    let url = format!("{}/api/v1/uc/claim_hty_resources", get_uc_url().trim_end_matches('/'));
    let resp = client
        .post(&url)
        .header("HtySudoerToken", sudoer)
        .header("Authorization", sudoer)
        .header("HtyHost", host)
        .json(&ClaimHtyResourcesReq { hty_resource_ids: ids })
        .send()
        .await?;
    let body = resp.json::<HtyResponse<usize>>().await?;
    if body.r {
        Ok(body.d.unwrap_or(0))
    } else {
        anyhow::bail!(
            "claim_hty_resources failed: {}",
            body.e.unwrap_or_else(|| "unknown".into())
        )
    }
}

/// 异步认领，不阻塞主请求（与 `remote_delete_by_id` 类似）。
pub fn uc_claim_hty_resources_spawn(
    hty_resource_ids: Vec<String>,
    claimer_hty_id: String,
    sudoer: String,
    host: String,
) {
    if hty_resource_ids.is_empty() || claimer_hty_id.is_empty() {
        return;
    }
    tokio::spawn(async move {
        match uc_claim_hty_resources(&hty_resource_ids, &claimer_hty_id, &sudoer, &host).await {
            Ok(n) => debug!(
                count = n,
                ids = ?hty_resource_ids,
                "uc_claim_hty_resources ok"
            ),
            Err(e) => tracing::warn!(
                error = %e,
                ids = ?hty_resource_ids,
                "uc_claim_hty_resources failed"
            ),
        }
    });
}
