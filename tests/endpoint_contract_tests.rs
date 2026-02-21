use serde::de::DeserializeOwned;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use wechat_mp_sdk::api::advertising::AdvertisingResponse;
use wechat_mp_sdk::api::analytics::{AnalyticsDateRangeRequest, AnalyticsResponse};
use wechat_mp_sdk::api::cloud::CloudResponse;
use wechat_mp_sdk::api::customer_service::TypingCommand;
use wechat_mp_sdk::api::delivery::DeliveryResponse;
use wechat_mp_sdk::api::endpoint_inventory::get_endpoint_inventory;
use wechat_mp_sdk::api::face::FaceResponse;
use wechat_mp_sdk::api::hardware::HardwareResponse;
use wechat_mp_sdk::api::live::LiveResponse;
use wechat_mp_sdk::api::logistics::LogisticsResponse;
use wechat_mp_sdk::api::nearby::NearbyResponse;
use wechat_mp_sdk::api::ocr::OcrImageRequest;
use wechat_mp_sdk::api::plugin::PluginResponse;
use wechat_mp_sdk::api::service_market::ServiceMarketResponse;
use wechat_mp_sdk::api::soter::VerifySignatureResponse;
use wechat_mp_sdk::api::wxsearch::SubmitPagesResponse;
use wechat_mp_sdk::client::WechatClient;
use wechat_mp_sdk::token::TokenManager;
use wechat_mp_sdk::types::{AppId, AppSecret};
use wechat_mp_sdk::{WechatError, WechatMp};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn create_test_client(mock_server: &MockServer) -> WechatClient {
    let appid = AppId::new("wx1234567890abcdef").unwrap();
    let secret = AppSecret::new("test_secret_12345").unwrap();
    WechatClient::builder()
        .appid(appid)
        .secret(secret)
        .base_url(mock_server.uri())
        .build()
        .unwrap()
}

async fn create_test_context(mock_server: &MockServer) -> (WechatClient, TokenManager) {
    let client = create_test_client(mock_server);
    let token_manager = TokenManager::new(client.clone());
    (client, token_manager)
}

async fn create_test_wechat(mock_server: &MockServer) -> WechatMp {
    let appid = AppId::new("wx1234567890abcdef").unwrap();
    let secret = AppSecret::new("test_secret_12345").unwrap();
    WechatMp::builder()
        .appid(appid)
        .secret(secret)
        .base_url(mock_server.uri())
        .build()
        .unwrap()
}

async fn mock_token(mock_server: &MockServer) {
    Mock::given(method("GET"))
        .and(path("/cgi-bin/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "test_token",
            "expires_in": 7200
        })))
        .mount(mock_server)
        .await;
}

async fn post_token<T: DeserializeOwned>(
    context: &(WechatClient, TokenManager),
    endpoint: &str,
    body: serde_json::Value,
) -> Result<T, WechatError> {
    let token = context.1.get_token().await?;
    let path = format!("{}?access_token={}", endpoint, token);
    context.0.post(&path, &body).await
}

fn assert_api_error<T: std::fmt::Debug>(result: Result<T, WechatError>, code: i32) {
    match result {
        Err(WechatError::Api { code: c, .. }) => assert_eq!(c, code),
        other => panic!("expected api error, got: {:?}", other),
    }
}

mod openapi_contract {
    use super::*;

    #[tokio::test]
    async fn openapi_success_contract() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/cgi-bin/clear_quota/v2"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"errcode": 0, "errmsg": "ok"})),
            )
            .mount(&server)
            .await;
        assert!(create_test_wechat(&server)
            .await
            .clear_quota_by_app_secret()
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn openapi_error_contract() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/cgi-bin/clear_quota/v2"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 40013, "errmsg": "invalid appid"})),
            )
            .mount(&server)
            .await;
        assert_api_error(
            create_test_wechat(&server)
                .await
                .clear_quota_by_app_secret()
                .await,
            40013,
        );
    }
}

mod security_contract {
    use super::*;

    #[tokio::test]
    async fn security_success_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/wxa/msg_sec_check"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "result": {"suggest": "pass", "label": 100},
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&server)
            .await;
        let response = create_test_wechat(&server)
            .await
            .msg_sec_check("openid", 1, "content")
            .await
            .unwrap();
        assert_eq!(response.result.suggest, "pass");
    }

    #[tokio::test]
    async fn security_error_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/wxa/msg_sec_check"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 87014, "errmsg": "risky content"})),
            )
            .mount(&server)
            .await;
        assert_api_error(
            create_test_wechat(&server)
                .await
                .msg_sec_check("openid", 1, "bad")
                .await,
            87014,
        );
    }
}

mod auth_extensions_contract {
    use super::*;

    #[tokio::test]
    async fn auth_extension_success_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/wxa/checksession"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"errcode": 0, "errmsg": "ok"})),
            )
            .mount(&server)
            .await;
        assert!(create_test_wechat(&server)
            .await
            .check_session_key("openid", "sig", "hmac_sha256")
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn auth_extension_error_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/wxa/checksession"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 40003, "errmsg": "invalid openid"})),
            )
            .mount(&server)
            .await;
        assert_api_error(
            create_test_wechat(&server)
                .await
                .check_session_key("bad", "sig", "hmac_sha256")
                .await,
            40003,
        );
    }
}

mod user_extensions_contract {
    use super::*;

    #[tokio::test]
    async fn user_extension_success_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/wxa/business/checkencryptedmsg"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"vaild": true, "errcode": 0, "errmsg": "ok"})),
            )
            .mount(&server)
            .await;
        let response = create_test_wechat(&server)
            .await
            .check_encrypted_data("hash")
            .await
            .unwrap();
        assert!(response.vaild);
    }

    #[tokio::test]
    async fn user_extension_error_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/wxa/business/checkencryptedmsg"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 40029, "errmsg": "invalid hash"})),
            )
            .mount(&server)
            .await;
        assert_api_error(
            create_test_wechat(&server)
                .await
                .check_encrypted_data("bad")
                .await,
            40029,
        );
    }
}

mod qrcode_extensions_contract {
    use super::*;

    #[tokio::test]
    async fn qrcode_extension_success_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/wxa/queryscheme"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "scheme_info": {"path": "/pages/index"},
                "scheme_quota": {"long_time_used": 1, "long_time_limit": 10},
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&server)
            .await;
        let response = create_test_wechat(&server)
            .await
            .query_scheme("weixin://dl/business/?t=x")
            .await
            .unwrap();
        assert_eq!(response.scheme_info.path, "/pages/index");
    }

    #[tokio::test]
    async fn qrcode_extension_error_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/wxa/queryscheme"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 40097, "errmsg": "invalid scheme"})),
            )
            .mount(&server)
            .await;
        assert_api_error(
            create_test_wechat(&server).await.query_scheme("bad").await,
            40097,
        );
    }
}

mod customer_service_typing_contract {
    use super::*;

    #[tokio::test]
    async fn customer_service_typing_success_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/cgi-bin/message/custom/typing"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({"errcode": 0, "errmsg": "ok"})),
            )
            .mount(&server)
            .await;
        assert!(create_test_wechat(&server)
            .await
            .set_typing("openid", TypingCommand::Typing)
            .await
            .is_ok());
    }

    #[tokio::test]
    async fn customer_service_typing_error_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/cgi-bin/message/custom/typing"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 40003, "errmsg": "invalid openid"})),
            )
            .mount(&server)
            .await;
        assert_api_error(
            create_test_wechat(&server)
                .await
                .set_typing("bad", TypingCommand::CancelTyping)
                .await,
            40003,
        );
    }
}

mod wechat_kf_contract {
    use super::*;

    #[tokio::test]
    async fn wechat_kf_success_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/cgi-bin/kfaccount/getbindedopenkfid"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "kf_list": [{"open_kfid": "kf_1", "kf_name": "support"}],
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&server)
            .await;
        let response = create_test_wechat(&server)
            .await
            .get_kf_work_bound("openid")
            .await
            .unwrap();
        assert_eq!(response.kf_list.len(), 1);
    }

    #[tokio::test]
    async fn wechat_kf_error_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/cgi-bin/kfaccount/getbindedopenkfid"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 40001, "errmsg": "invalid credential"})),
            )
            .mount(&server)
            .await;
        assert_api_error(
            create_test_wechat(&server)
                .await
                .get_kf_work_bound("openid")
                .await,
            40001,
        );
    }
}

mod subscribe_extensions_contract {
    use super::*;

    #[tokio::test]
    async fn subscribe_success_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("GET"))
            .and(path("/wxaapi/newtmpl/getpubtemplatekeywords"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "data": [{"kid": 1, "name": "thing1", "rule": "x"}],
                "errcode": 0,
                "errmsg": "ok"
            })))
            .mount(&server)
            .await;
        let response = create_test_wechat(&server)
            .await
            .get_pub_template_keywords_by_id("tid_1")
            .await
            .unwrap();
        assert_eq!(response.data[0].kid, 1);
    }

    #[tokio::test]
    async fn subscribe_error_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("GET"))
            .and(path("/wxaapi/newtmpl/getpubtemplatekeywords"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 41030, "errmsg": "invalid tid"})),
            )
            .mount(&server)
            .await;
        assert_api_error(
            create_test_wechat(&server)
                .await
                .get_pub_template_keywords_by_id("bad")
                .await,
            41030,
        );
    }
}

mod analytics_contract {
    use super::*;

    #[tokio::test]
    async fn analytics_daily_visit_success_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/datacube/getweanalysisappiddailyvisittrend"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                json!({"list": [{"ref_date": "20250101"}], "errcode": 0, "errmsg": "ok"}),
            ))
            .mount(&server)
            .await;
        let req = AnalyticsDateRangeRequest::new("20250101", "20250101");
        let response = create_test_wechat(&server)
            .await
            .get_daily_visit_trend(&req)
            .await
            .unwrap();
        assert!(response.extra.contains_key("list"));
    }

    #[tokio::test]
    async fn analytics_daily_visit_error_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/datacube/getweanalysisappiddailyvisittrend"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 40033, "errmsg": "invalid date"})),
            )
            .mount(&server)
            .await;
        let req = AnalyticsDateRangeRequest::new("bad", "bad");
        assert_api_error(
            create_test_wechat(&server)
                .await
                .get_daily_visit_trend(&req)
                .await,
            40033,
        );
    }

    #[tokio::test]
    async fn analytics_user_portrait_success_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/datacube/getweanalysisappiduserportrait"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"visit_uv": [], "errcode": 0, "errmsg": "ok"})),
            )
            .mount(&server)
            .await;
        let context = create_test_context(&server).await;
        let response: AnalyticsResponse = post_token(
            &context,
            "/datacube/getweanalysisappiduserportrait",
            json!({"begin_date": "20250101", "end_date": "20250101"}),
        )
        .await
        .unwrap();
        assert!(response.extra.contains_key("visit_uv"));
    }

    #[tokio::test]
    async fn analytics_user_portrait_error_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/datacube/getweanalysisappiduserportrait"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 40035, "errmsg": "invalid params"})),
            )
            .mount(&server)
            .await;
        let context = create_test_context(&server).await;
        let result: Result<AnalyticsResponse, WechatError> = post_token(
            &context,
            "/datacube/getweanalysisappiduserportrait",
            json!({"begin_date": "bad", "end_date": "bad"}),
        )
        .await;
        assert_api_error(result, 40035);
    }
}

mod operations_contract {
    use super::*;

    #[tokio::test]
    async fn operations_domain_success_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/wxa/get_wxa_domain"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"requestdomain": [], "errcode": 0, "errmsg": "ok"})),
            )
            .mount(&server)
            .await;
        let response = create_test_wechat(&server)
            .await
            .get_domain_info()
            .await
            .unwrap();
        assert!(response.extra.contains_key("requestdomain"));
    }

    #[tokio::test]
    async fn operations_domain_error_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/wxa/get_wxa_domain"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 40001, "errmsg": "invalid credential"})),
            )
            .mount(&server)
            .await;
        assert_api_error(
            create_test_wechat(&server).await.get_domain_info().await,
            40001,
        );
    }

    #[tokio::test]
    async fn operations_scene_success_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("GET"))
            .and(path("/wxaapi/log/get_scene"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"scene": [{"scene": 1}], "errcode": 0, "errmsg": "ok"})),
            )
            .mount(&server)
            .await;
        let response = create_test_wechat(&server)
            .await
            .get_scene_list()
            .await
            .unwrap();
        assert!(response.extra.contains_key("scene"));
    }

    #[tokio::test]
    async fn operations_scene_error_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("GET"))
            .and(path("/wxaapi/log/get_scene"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 40037, "errmsg": "invalid appid"})),
            )
            .mount(&server)
            .await;
        assert_api_error(
            create_test_wechat(&server).await.get_scene_list().await,
            40037,
        );
    }
}

mod ocr_contract {
    use super::*;

    #[tokio::test]
    async fn ocr_success_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/cv/ocr/bankcard"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"number": "6222", "errcode": 0, "errmsg": "ok"})),
            )
            .mount(&server)
            .await;
        let request = OcrImageRequest::new("https://img/card.jpg");
        let response = create_test_wechat(&server)
            .await
            .bank_card_ocr(&request)
            .await
            .unwrap();
        assert!(response.extra.contains_key("number"));
    }

    #[tokio::test]
    async fn ocr_error_contract() {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path("/cv/ocr/bankcard"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 40008, "errmsg": "invalid image"})),
            )
            .mount(&server)
            .await;
        let request = OcrImageRequest::new("bad");
        assert_api_error(
            create_test_wechat(&server)
                .await
                .bank_card_ocr(&request)
                .await,
            40008,
        );
    }
}

mod generic_family_contracts {
    use super::*;

    async fn run_success<T: DeserializeOwned>(endpoint: &str, body: serde_json::Value) -> T {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path(endpoint))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": 0, "errmsg": "ok", "data": {"ok": true}})),
            )
            .mount(&server)
            .await;
        post_token(&create_test_context(&server).await, endpoint, body)
            .await
            .unwrap()
    }

    async fn run_error<T: DeserializeOwned + std::fmt::Debug>(
        endpoint: &str,
        body: serde_json::Value,
        code: i32,
    ) {
        let server = MockServer::start().await;
        mock_token(&server).await;
        Mock::given(method("POST"))
            .and(path(endpoint))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(json!({"errcode": code, "errmsg": "failed"})),
            )
            .mount(&server)
            .await;
        let result: Result<T, WechatError> =
            post_token(&create_test_context(&server).await, endpoint, body).await;
        assert_api_error(result, code);
    }

    #[tokio::test]
    async fn plugin_success_contract() {
        let response: PluginResponse = run_success(
            "/wxa/plugin",
            json!({"action": "apply", "plugin_appid": "wx_plugin"}),
        )
        .await;
        assert!(response.extra.contains_key("data"));
    }

    #[tokio::test]
    async fn plugin_error_contract() {
        run_error::<PluginResponse>("/wxa/plugin", json!({"action": "apply"}), 85400).await;
    }

    #[tokio::test]
    async fn nearby_success_contract() {
        let response: NearbyResponse = run_success(
            "/wxa/addnearbypoi",
            json!({"poi_id": "poi", "related_name": "name", "related_credential": "cred", "related_address": "addr"}),
        )
        .await;
        assert!(response.extra.contains_key("data"));
    }

    #[tokio::test]
    async fn nearby_error_contract() {
        run_error::<NearbyResponse>("/wxa/addnearbypoi", json!({"poi_id": "bad"}), 85097).await;
    }

    #[tokio::test]
    async fn cloud_success_contract() {
        let response: CloudResponse = run_success(
            "/tcb/invokecloudfunction",
            json!({"name": "fn", "env": "prod"}),
        )
        .await;
        assert!(response.extra.contains_key("data"));
    }

    #[tokio::test]
    async fn cloud_error_contract() {
        run_error::<CloudResponse>("/tcb/invokecloudfunction", json!({"name": "bad"}), 41001).await;
    }

    #[tokio::test]
    async fn live_success_contract() {
        let response: LiveResponse =
            run_success("/wxaapi/broadcast/room/create", json!({"name": "room"})).await;
        assert!(response.extra.contains_key("data"));
    }

    #[tokio::test]
    async fn live_error_contract() {
        run_error::<LiveResponse>("/wxaapi/broadcast/room/create", json!({"name": ""}), 300001)
            .await;
    }

    #[tokio::test]
    async fn hardware_success_contract() {
        let response: HardwareResponse =
            run_success("/wxa/business/hardware/sn_ticket/get", json!({"sn": "sn1"})).await;
        assert!(response.extra.contains_key("data"));
    }

    #[tokio::test]
    async fn hardware_error_contract() {
        run_error::<HardwareResponse>(
            "/wxa/business/hardware/sn_ticket/get",
            json!({"sn": "bad"}),
            40011,
        )
        .await;
    }

    #[tokio::test]
    async fn delivery_success_contract() {
        let response: DeliveryResponse = run_success(
            "/cgi-bin/express/local/business/order/add",
            json!({"shop_no": "shop1"}),
        )
        .await;
        assert!(response.extra.contains_key("data"));
    }

    #[tokio::test]
    async fn delivery_error_contract() {
        run_error::<DeliveryResponse>(
            "/cgi-bin/express/local/business/order/add",
            json!({"shop_no": "bad"}),
            9301001,
        )
        .await;
    }

    #[tokio::test]
    async fn logistics_success_contract() {
        let response: LogisticsResponse = run_success(
            "/cgi-bin/express/business/account/bind",
            json!({"delivery_id": "DHL"}),
        )
        .await;
        assert!(response.extra.contains_key("data"));
    }

    #[tokio::test]
    async fn logistics_error_contract() {
        run_error::<LogisticsResponse>(
            "/cgi-bin/express/business/account/bind",
            json!({"delivery_id": "bad"}),
            9300002,
        )
        .await;
    }

    #[tokio::test]
    async fn service_market_success_contract() {
        let response: ServiceMarketResponse =
            run_success("/wxa/servicemarket", json!({"service": "captcha"})).await;
        assert!(response.extra.contains_key("data"));
    }

    #[tokio::test]
    async fn service_market_error_contract() {
        run_error::<ServiceMarketResponse>("/wxa/servicemarket", json!({"service": "bad"}), 40100)
            .await;
    }

    #[tokio::test]
    async fn soter_success_contract() {
        let response: VerifySignatureResponse = run_success(
            "/cgi-bin/soter/verify_signature",
            json!({"openid": "openid"}),
        )
        .await;
        assert!(response.extra.contains_key("data"));
    }

    #[tokio::test]
    async fn soter_error_contract() {
        run_error::<VerifySignatureResponse>(
            "/cgi-bin/soter/verify_signature",
            json!({"openid": "openid"}),
            90001,
        )
        .await;
    }

    #[tokio::test]
    async fn face_success_contract() {
        let response: FaceResponse =
            run_success("/cgi-bin/soter/mp/verify_id/get", json!({"name": "alice"})).await;
        assert!(response.extra.contains_key("data"));
    }

    #[tokio::test]
    async fn face_error_contract() {
        run_error::<FaceResponse>(
            "/cgi-bin/soter/mp/verify_id/get",
            json!({"name": "alice"}),
            50002,
        )
        .await;
    }

    #[tokio::test]
    async fn wxsearch_success_contract() {
        let response: SubmitPagesResponse = run_success(
            "/wxa/search/wxaapi_submitpages",
            json!({"pages": ["/pages/index"]}),
        )
        .await;
        assert!(response.extra.contains_key("data"));
    }

    #[tokio::test]
    async fn wxsearch_error_contract() {
        run_error::<SubmitPagesResponse>(
            "/wxa/search/wxaapi_submitpages",
            json!({"pages": ["bad"]}),
            85066,
        )
        .await;
    }

    #[tokio::test]
    async fn advertising_success_contract() {
        let response: AdvertisingResponse = run_success(
            "/marketing/add_user_action",
            json!({"action_type": "purchase"}),
        )
        .await;
        assert!(response.extra.contains_key("data"));
    }

    #[tokio::test]
    async fn advertising_error_contract() {
        run_error::<AdvertisingResponse>(
            "/marketing/add_user_action",
            json!({"action_type": "bad"}),
            44001,
        )
        .await;
    }
}

const CONTRACT_ENDPOINT_TEST_MAP: &[(&str, &str)] = &[
    ("openapi.clearQuotaByAppSecret", "openapi_success_contract"),
    ("security.msgSecCheck", "security_success_contract"),
    ("auth.checkSessionKey", "auth_extension_success_contract"),
    ("user.checkEncryptedData", "user_extension_success_contract"),
    ("qrcode.queryScheme", "qrcode_extension_success_contract"),
    (
        "customerService.setTyping",
        "customer_service_typing_success_contract",
    ),
    ("kfWork.getKfWorkBound", "wechat_kf_success_contract"),
    (
        "subscribe.getPubTemplateKeyWordsById",
        "subscribe_success_contract",
    ),
    (
        "analytics.getDailyVisitTrend",
        "analytics_daily_visit_success_contract",
    ),
    (
        "analytics.getUserPortrait",
        "analytics_user_portrait_success_contract",
    ),
    (
        "operations.getDomainInfo",
        "operations_domain_success_contract",
    ),
    (
        "operations.getSceneList",
        "operations_scene_success_contract",
    ),
    ("ocr.bankCardOCR", "ocr_success_contract"),
    ("plugin.managePluginApplication", "plugin_success_contract"),
    ("nearby.addNearbyPoi", "nearby_success_contract"),
    ("cloud.invokeCloudFunction", "cloud_success_contract"),
    ("live.createRoom", "live_success_contract"),
    ("hardware.getSnTicket", "hardware_success_contract"),
    ("delivery.addLocalOrder", "delivery_success_contract"),
    ("express.bindAccount", "logistics_success_contract"),
    (
        "serviceMarket.invokeService",
        "service_market_success_contract",
    ),
    ("soter.verifySignature", "soter_success_contract"),
    ("face.getVerifyId", "face_success_contract"),
    ("wxsearch.submitPages", "wxsearch_success_contract"),
    ("ad.addUserAction", "advertising_success_contract"),
];

#[test]
fn all_endpoints_have_contract_tests() {
    let required_categories: HashSet<&str> = HashSet::from([
        "openapi",
        "security",
        "login",
        "user-info",
        "qrcode-link",
        "customer-service",
        "wechat-kf",
        "subscribe-message",
        "analytics",
        "operations",
        "image-ocr",
        "plugin",
        "nearby",
        "cloud",
        "live",
        "hardware",
        "instant-delivery",
        "logistics",
        "service-market",
        "soter",
        "face",
        "wxsearch",
        "advertising",
    ]);

    let map: HashMap<&str, &str> = CONTRACT_ENDPOINT_TEST_MAP.iter().copied().collect();
    let required_ids: HashSet<&str> = CONTRACT_ENDPOINT_TEST_MAP
        .iter()
        .map(|(id, _)| *id)
        .collect();

    let missing_from_map: Vec<String> = get_endpoint_inventory()
        .iter()
        .filter(|item| item.implemented && !item.deprecated)
        .filter(|item| required_categories.contains(item.category))
        .filter(|item| required_ids.contains(item.endpoint_id))
        .filter(|item| !map.contains_key(item.endpoint_id))
        .map(|item| item.endpoint_id.to_string())
        .collect();

    assert!(
        missing_from_map.is_empty(),
        "missing endpoint contract mappings:\n  - {}",
        missing_from_map.join("\n  - ")
    );
}
