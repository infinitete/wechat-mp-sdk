//! Analytics API

use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{WechatApi, WechatContext};
use crate::error::WechatError;

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct AnalyticsDateRangeRequest {
    pub begin_date: String,
    pub end_date: String,
}

impl AnalyticsDateRangeRequest {
    pub fn new(begin_date: impl Into<String>, end_date: impl Into<String>) -> Self {
        Self {
            begin_date: begin_date.into(),
            end_date: end_date.into(),
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Serialize)]
pub struct PerformanceDataRequest {
    pub cost_time_type: i32,
    pub default_start_time: i64,
    pub default_end_time: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networktype: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene: Option<i32>,
}

#[non_exhaustive]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnalyticsResponse {
    #[serde(default)]
    pub(crate) errcode: i32,
    #[serde(default)]
    pub(crate) errmsg: String,
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

pub struct AnalyticsApi {
    context: Arc<WechatContext>,
}

impl AnalyticsApi {
    pub fn new(context: Arc<WechatContext>) -> Self {
        Self { context }
    }

    pub async fn get_daily_summary(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        self.post_datacube("/datacube/getweanalysisappiddailysummarytrend", request)
            .await
    }

    pub async fn get_daily_visit_trend(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        self.post_datacube("/datacube/getweanalysisappiddailyvisittrend", request)
            .await
    }

    pub async fn get_weekly_visit_trend(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        self.post_datacube("/datacube/getweanalysisappidweeklyvisittrend", request)
            .await
    }

    pub async fn get_monthly_visit_trend(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        self.post_datacube("/datacube/getweanalysisappidmonthlyvisittrend", request)
            .await
    }

    pub async fn get_daily_retain(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        self.post_datacube("/datacube/getweanalysisappiddailyretaininfo", request)
            .await
    }

    pub async fn get_weekly_retain(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        self.post_datacube("/datacube/getweanalysisappidweeklyretaininfo", request)
            .await
    }

    pub async fn get_monthly_retain(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        self.post_datacube("/datacube/getweanalysisappidmonthlyretaininfo", request)
            .await
    }

    pub async fn get_visit_page(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        self.post_datacube("/datacube/getweanalysisappidvisitpage", request)
            .await
    }

    pub async fn get_visit_distribution(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        self.post_datacube("/datacube/getweanalysisappidvisitdistribution", request)
            .await
    }

    pub async fn get_user_portrait(
        &self,
        request: &AnalyticsDateRangeRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        self.post_datacube("/datacube/getweanalysisappiduserportrait", request)
            .await
    }

    pub async fn get_performance_data(
        &self,
        request: &PerformanceDataRequest,
    ) -> Result<AnalyticsResponse, WechatError> {
        self.post_datacube("/wxaapi/log/get_performance", request)
            .await
    }

    async fn post_datacube<B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<AnalyticsResponse, WechatError> {
        let access_token = self.context.token_manager.get_token().await?;
        let path = format!("{}?access_token={}", endpoint, access_token);
        let response: AnalyticsResponse = self.context.client.post(&path, body).await?;
        WechatError::check_api(response.errcode, &response.errmsg)?;
        Ok(response)
    }
}

impl WechatApi for AnalyticsApi {
    fn context(&self) -> &WechatContext {
        &self.context
    }

    fn api_name(&self) -> &'static str {
        "analytics"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analytics_response_deserializes() {
        let json = r#"{"errcode":0,"errmsg":"ok","list":[{"k":"v"}]}"#;
        let response: AnalyticsResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.errcode, 0);
        assert!(response.extra.contains_key("list"));
    }

    #[test]
    fn analytics_date_range_request_serializes() {
        let request = AnalyticsDateRangeRequest::new("20240101", "20240102");
        let value = serde_json::to_value(request).unwrap();
        assert_eq!(value["begin_date"], "20240101");
        assert_eq!(value["end_date"], "20240102");
    }
}
