use wechat_mp_sdk::{
    types::{AppId, AppSecret},
    WechatMp,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let wechat = WechatMp::builder()
        .appid(AppId::new("wx1234567890abcdef")?)
        .secret(AppSecret::new("your_app_secret_here")?)
        .build()?;

    match wechat.get_domain_info().await {
        Ok(response) => println!("Domain info keys: {}", response.extra.len()),
        Err(error) => eprintln!("get_domain_info failed: {error}"),
    }

    match wechat.get_scene_list().await {
        Ok(response) => println!("Scene list keys: {}", response.extra.len()),
        Err(error) => eprintln!("get_scene_list failed: {error}"),
    }

    match wechat.get_api_domain_ip().await {
        Ok(response) => println!("API domain IP count: {}", response.ip_list.len()),
        Err(error) => eprintln!("get_api_domain_ip failed: {error}"),
    }

    match wechat.get_callback_ip().await {
        Ok(response) => println!("Callback IP count: {}", response.ip_list.len()),
        Err(error) => eprintln!("get_callback_ip failed: {error}"),
    }

    Ok(())
}
