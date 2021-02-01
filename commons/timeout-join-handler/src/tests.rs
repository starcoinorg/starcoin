use super::*;
use anyhow::anyhow;

#[test]
fn test_ok() {
    let handle = spawn(|| {
        thread::sleep(Duration::from_millis(100));
        let result: Result<u64, anyhow::Error> = Ok(1);
        result
    });
    let result = handle.join(Duration::from_millis(1000));
    assert!(result.is_ok());
    assert_eq!(1, result.unwrap().unwrap());
}

#[test]
fn test_error_in_thread() {
    let handle = spawn(|| {
        thread::sleep(Duration::from_millis(100));
        let result: Result<(), anyhow::Error> = Err(anyhow!("anyhow error"));
        result
    });
    let result = handle.join(Duration::from_millis(1000));
    assert!(result.is_ok());
    assert!(result.unwrap().is_err());
}

#[test]
fn test_timeout() {
    let handle = spawn(|| {
        thread::sleep(Duration::from_secs(1));
    });
    let result = handle.join(Duration::from_millis(100));
    assert!(result.is_err());
    let error = result.err().unwrap();
    assert!(error.is_timeout());
    let handle = error.into_handle().unwrap();
    let result = handle.join(Duration::from_secs(2));
    assert!(result.is_ok());
}

#[test]
fn test_panic() {
    let handle = spawn(|| {
        panic!("test thread panic");
    });
    let result = handle.join(Duration::from_secs(2));
    let error = result.err().unwrap();
    assert!(error.is_panic());
    assert_eq!(error.panic_message().unwrap(), "test thread panic");
}
