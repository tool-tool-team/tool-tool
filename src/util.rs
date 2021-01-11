use crate::Result;
use anyhow::Context;
use std::time::Duration;


// NOTE: under windows, file operations often fail spuriously, most likely due to virus scanners
// keeping files open
// This retry schedule should hopefully handle most situations
const RETRY_DURATIONS: &[Duration] = &[
    Duration::from_millis(0),
    Duration::from_millis(1),
    Duration::from_millis(10),
    Duration::from_millis(100),
    Duration::from_millis(1000),
    Duration::from_millis(1000),
    Duration::from_millis(1000),
    Duration::from_millis(1000),
    Duration::from_millis(1000),
    Duration::from_millis(1000),
    Duration::from_millis(1000),
    Duration::from_millis(1000),
    Duration::from_millis(1000),
    Duration::from_millis(1000),
];

pub fn retry<T, E, F>(mut func: F) -> Result<T>
where
    F: FnMut() -> std::result::Result<T, E>,
    E: std::error::Error + Send + Sync + 'static,
{
    let mut tries = 0;
    loop {
        let result = func();
        match result {
            Ok(res) => return Ok(res),
            Err(err) => {
                if tries < RETRY_DURATIONS.len() {
                    std::thread::sleep(RETRY_DURATIONS[tries])
                } else {
                    let context = format!("failed after {} tries: {}", tries, err);
                    return Err(err).context(context);
                }
            }
        }
        tries += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retry_success() -> Result<()> {
        let res = retry::<_, std::io::Error, _>(|| Ok(42))?;
        assert_eq!(res, 42);
        Ok(())
    }

    #[test]
    fn retry_failure() -> Result<()> {
        let res = retry::<u32, _, _>(|| Err(std::env::VarError::NotPresent));
        assert!(res.is_err());
        assert_eq!(
            "failed after 5 tries: environment variable not found".to_string(),
            res.unwrap_err().to_string()
        );
        Ok(())
    }

    #[test]
    fn retry_fail_then_succeed() -> Result<()> {
        let mut invocations = 0;
        let res = retry::<i32, _, _>(|| {
            invocations += 1;
            if invocations < 4 {
                Err(std::env::VarError::NotPresent)
            } else {
                Ok(invocations)
            }
        })?;
        assert_eq!(res, 4);
        Ok(())
    }
}
