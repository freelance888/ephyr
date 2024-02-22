use srs_client::{SrsClient, SrsClientError, SrsClientResp};
use std::env;
use tokio;

// #[tokio::test]
// async fn test_kickoff_client() -> Result<(), Box<dyn std::error::Error>> {
//     let srs_http_api_url =
//         env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
//     let client = SrsClient::build(&srs_http_api_url)?;
//     let result: Result<SrsClientResp, SrsClientError> =
//         client.kickoff_client("21233").await;
//     assert!(result.is_ok());
//     Ok(())
// }

#[tokio::test]
async fn test_get_version() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url =
        env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> =
        client.get_version().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_get_vhosts() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url =
        env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> =
        client.get_vhosts().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_get_streams() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url =
        env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> =
        client.get_streams().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_get_clients() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url =
        env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> =
        client.get_clients().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_get_features() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url =
        env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> =
        client.get_features().await;
    assert!(result.is_ok());
    Ok(())
}

#[tokio::test]
async fn test_get_rusages() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url =
        env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> =
        client.get_rusages().await;
    assert!(result.is_ok());
    Ok(())
}
#[tokio::test]
async fn test_get_self_proc_stats() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url =
        env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> =
        client.get_self_proc_stats().await;
    assert!(result.is_ok());
    Ok(())
}
#[tokio::test]
async fn test_get_system_proc_stats() -> Result<(), Box<dyn std::error::Error>>
{
    let srs_http_api_url =
        env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> =
        client.get_system_proc_stats().await;
    assert!(result.is_ok());
    Ok(())
}
#[tokio::test]
async fn test_get_meminfos() -> Result<(), Box<dyn std::error::Error>> {
    let srs_http_api_url =
        env::var("SRS_HTTP_API_URL").expect("SRS_HTTP_API_URL not set");
    let client = SrsClient::build(&srs_http_api_url)?;
    let result: Result<SrsClientResp, SrsClientError> =
        client.get_meminfos().await;
    assert!(result.is_ok());
    Ok(())
}
