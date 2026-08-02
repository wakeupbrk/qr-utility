use qr_utility::generator::QrGenerator;
use qr_utility::models::{EccLevel, ExpirationOption, ExportFormat, QrItem, TargetType, TimeUnit};
use qr_utility::services::{LocalRedirectProvider, RedirectProvider, RedirectResolution};
use qr_utility::storage::HistoryStore;
use qr_utility::utils::UrlValidator;

#[tokio::test]
async fn test_url_validation() {
    assert!(UrlValidator::validate("https://example.com").is_ok());
    assert!(UrlValidator::validate("http://google.com").is_ok());
    assert!(UrlValidator::validate("github.com").is_ok());
    assert!(UrlValidator::validate("").is_err());
}

#[tokio::test]
async fn test_expiration_options() {
    let opt_5m = ExpirationOption::FiveMinutes;
    assert_eq!(opt_5m.label(), "5 minutes");
    assert!(opt_5m.to_timestamp().is_some());

    let never = ExpirationOption::Never;
    assert_eq!(never.label(), "Never expires");
    assert!(never.to_timestamp().is_none());

    let custom = ExpirationOption::from_str_lenient("45m").unwrap();
    assert_eq!(
        custom,
        ExpirationOption::Custom {
            value: 45,
            unit: TimeUnit::Minutes
        }
    );
}

#[tokio::test]
async fn test_qr_generation() {
    let qr = QrGenerator::create_qr("https://example.com", EccLevel::Medium).unwrap();
    let unicode_art = QrGenerator::render_unicode(&qr, true);
    assert!(!unicode_art.is_empty());
    assert!(unicode_art.contains('█') || unicode_art.contains('▀') || unicode_art.contains('▄'));
}

#[tokio::test]
async fn test_redirect_provider() {
    // HistoryStore is now Arc-backed internally — cloning shares data
    let store = HistoryStore::default();
    let provider = LocalRedirectProvider::new(store.clone());

    let url = "https://example.com".to_string();
    let item = QrItem::new(
        url.clone(),
        TargetType::Url(url),
        "testcode".to_string(),
        None,
        ExpirationOption::FiveMinutes.to_timestamp(),
        EccLevel::Medium,
        ExportFormat::Png,
    );

    store.add(item).unwrap();

    let res = provider.resolve_short_code("testcode").await;
    match res {
        RedirectResolution::ActiveUrl { target_url } => {
            assert_eq!(target_url, "https://example.com");
        }
        _ => panic!("Expected active URL redirect resolution"),
    }
}
