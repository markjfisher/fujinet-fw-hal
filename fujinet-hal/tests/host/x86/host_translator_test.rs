use fujinet_hal::host::{HostTranslator, X86HostTranslator};

#[tokio::test]
async fn test_x86_host_translator() {
    let mut translator = X86HostTranslator::new();
    
    // Test initialization
    assert!(translator.initialize().await.is_ok());
    
    // Test data pass-through
    let test_data = b"Hello, World!";
    let result = translator.process_host_data(test_data).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_ref(), test_data);
    
    let result = translator.process_device_data(test_data).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().as_ref(), test_data);
}