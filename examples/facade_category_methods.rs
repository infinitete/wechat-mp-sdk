use wechat_mp_sdk::{
    api::{AnalyticsDateRangeRequest, OcrImageRequest},
    types::{AppId, AppSecret},
    WechatMp,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wechat = WechatMp::builder()
        .appid(AppId::new("wx1234567890abcdef")?)
        .secret(AppSecret::new("your_app_secret_here")?)
        .build()?;

    let analytics_range = AnalyticsDateRangeRequest::new("20240101", "20240101");
    match wechat.get_daily_summary(&analytics_range).await {
        Ok(response) => println!("Daily summary keys: {}", response.extra.len()),
        Err(error) => eprintln!("get_daily_summary failed: {error}"),
    }

    let ocr_request = OcrImageRequest::new("https://example.com/id-card.jpg");
    match wechat.scan_qr_code(&ocr_request).await {
        Ok(response) => println!("OCR response keys: {}", response.extra.len()),
        Err(error) => eprintln!("scan_qr_code failed: {error}"),
    }

    let _ = WechatMp::database_query;
    let _ = WechatMp::create_room;
    let _ = WechatMp::add_local_order;
    let _ = WechatMp::add_order;

    Ok(())
}
