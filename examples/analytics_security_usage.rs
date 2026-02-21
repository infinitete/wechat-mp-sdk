use wechat_mp_sdk::{
    api::AnalyticsDateRangeRequest,
    types::{AppId, AppSecret},
    WechatMp,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wechat = WechatMp::builder()
        .appid(AppId::new("wx1234567890abcdef")?)
        .secret(AppSecret::new("your_app_secret_here")?)
        .build()?;

    let analytics_range = AnalyticsDateRangeRequest::new("20240101", "20240107");

    match wechat.get_daily_visit_trend(&analytics_range).await {
        Ok(response) => println!("Daily visit trend keys: {}", response.extra.len()),
        Err(error) => eprintln!("get_daily_visit_trend failed: {error}"),
    }

    match wechat
        .msg_sec_check("o_user_openid_123456789012345678", 1, "safe content")
        .await
    {
        Ok(response) => println!("Message security suggestion: {}", response.result.suggest),
        Err(error) => eprintln!("msg_sec_check failed: {error}"),
    }

    match wechat
        .get_user_risk_rank("o_user_openid_123456789012345678", 0, None)
        .await
    {
        Ok(response) => println!("User risk rank: {}", response.risk_rank),
        Err(error) => eprintln!("get_user_risk_rank failed: {error}"),
    }

    Ok(())
}
