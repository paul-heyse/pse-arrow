// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Explicit platform path data. Native adoption does no filesystem work.
use super::*;
use std::path::PathBuf;

crate::projection_enum! {
    /// Lossless foreign path observations remain inspectable independently of native adoption.
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum PlatformPathBytes {
        projection: { name: "PlatformPathBytes", rust_path: crate::boundary_projection::PlatformPathBytes,
            version: 1, direction: Direction::Both, reserved_tags: [], reserved_names: [] };
        /// Unix OsStr bytes, without UTF-8 inference or normalization.
        Unix(Vec<u8>) => 1;
        /// Windows OsStr wide units, including unpaired surrogates.
        Windows(Vec<u16>) => 2;
    }
}
impl ProjectionType for PathBuf {
    fn projection(graph: &mut ProjectionGraph) -> Result<TypeRef, ProjectionError> {
        let root = graph.declare::<Self>(
            "NativePathBuf",
            file!(),
            1,
            Direction::Both,
            &[],
            &[],
            |graph| {
                Ok(Shape::Record(vec![Field::of::<PlatformPathBytes>(
                    graph,
                    "platform",
                    1,
                    Direction::Both,
                    vec![LocalRule::Required],
                )?]))
            },
        )?;
        graph.public_path::<Self>(&root, "std::path::PathBuf")?;
        graph.checked_record::<Self>(&root)?;
        Ok(root)
    }
}
impl ValidateProjection for PathBuf {
    fn validate_at(&self, _: &str) -> Result<(), ValidationError> {
        Ok(())
    }
}
impl CheckedProjectionRecord for PathBuf {
    const VERSION: u32 = 1;
    const FIELD_LAYOUT: &'static [(u32, &'static str)] = &[(1, "platform")];
    type Fields = (PlatformPathBytes,);
    fn try_into_projection_fields(self) -> Result<Self::Fields, AdapterError> {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStringExt;
            Ok((PlatformPathBytes::Unix(self.into_os_string().into_vec()),))
        }
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            Ok((PlatformPathBytes::Windows(
                self.as_os_str().encode_wide().collect(),
            ),))
        }
        #[cfg(not(any(unix, windows)))]
        {
            Err(AdapterError::refused(
                "platform",
                AdapterFailure::InvalidValue,
                "unsupported native path platform",
            ))
        }
    }
    fn from_projection_fields((platform,): Self::Fields) -> Result<Self, AdapterError> {
        match platform {
            #[cfg(unix)]
            PlatformPathBytes::Unix(bytes) => {
                use std::os::unix::ffi::OsStringExt;
                Ok(std::ffi::OsString::from_vec(bytes).into())
            }
            #[cfg(windows)]
            PlatformPathBytes::Windows(units) => {
                use std::os::windows::ffi::OsStringExt;
                Ok(std::ffi::OsString::from_wide(&units).into())
            }
            _ => Err(AdapterError::refused(
                "platform",
                AdapterFailure::InvalidValue,
                "foreign platform path data cannot become a native PathBuf",
            )),
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::ffi::OsStringExt;
    #[test]
    fn native_path_preserves_exact_bytes_and_refuses_foreign_platform_adoption() {
        let bytes = vec![b'/', b't', 0xff, 0, b'x'];
        let path = PathBuf::from(std::ffi::OsString::from_vec(bytes.clone()));
        let fields = path.clone().try_into_projection_fields().unwrap();
        assert_eq!(fields.0, PlatformPathBytes::Unix(bytes));
        assert_eq!(PathBuf::from_projection_fields(fields).unwrap(), path);
        let foreign = PlatformPathBytes::Windows(vec![0xd800, 0x005c]);
        foreign.validate_projection().unwrap();
        assert!(PathBuf::from_projection_fields((foreign,)).is_err());
        let graph = ProjectionGraph::of::<PathBuf>().unwrap();
        assert!(graph.definitions()["NativePathBuf"].checked_record);
    }
}
