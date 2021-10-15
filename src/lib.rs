include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::ptr::addr_of_mut;

    type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
    #[test]
    fn test_versioninfo() {
        unsafe {
            ShowVersionInfo();
        }
    }

    #[test]
    fn test_basepath() -> Result<()> {
        let input = CString::new("foo.bar.c")?;
        let mut basepath: Vec<i8> = vec!['\0' as i8; input.to_bytes().len()];
        let mut baseptr = basepath.as_mut_ptr();
        let result = unsafe {
            GetBasePath(input.as_ptr(), addr_of_mut!(baseptr));
            CStr::from_ptr(baseptr)
        };
        assert_eq!(result.to_str()?, "foo.bar");
        Ok(())
    }
}
