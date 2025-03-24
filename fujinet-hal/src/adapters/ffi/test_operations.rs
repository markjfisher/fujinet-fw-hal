use std::sync::{Mutex, OnceLock};
use crate::adapters::common::network::operations::OperationsContext;
use crate::adapters::common::network::test_mocks::TestNetworkManager;
use crate::device::network::manager::NetworkManagerImpl;

// Test-specific operations context
static TEST_OPERATIONS: OnceLock<Mutex<Option<OperationsContext<TestNetworkManager>>>> = OnceLock::new();

pub fn setup_test_context(manager: TestNetworkManager) {
    let _ = TEST_OPERATIONS.get_or_init(|| Mutex::new(None));
    *TEST_OPERATIONS.get().unwrap().lock().unwrap() = Some(OperationsContext::new(manager));
}

// Override OPERATIONS for tests
pub fn get_test_operations() -> &'static OperationsContext<NetworkManagerImpl> {
    let test_ops = TEST_OPERATIONS.get().unwrap().lock().unwrap();
    // This is safe because we're in tests and we control the static lifetime
    unsafe { std::mem::transmute(test_ops.as_ref().unwrap()) }
} 