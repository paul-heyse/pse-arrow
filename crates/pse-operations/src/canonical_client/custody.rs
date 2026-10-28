// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse

//! Non-byte custody of already issued operational guards. No attribution,
//! identifier or allocation can construct a read capability through this carrier.
use crate::{
    canonical::ProtectedSelection,
    canonical_artifacts::ArtifactRead,
    canonical_capture_results::CapturedResultRead,
    canonical_results::{CompletionRead, ResultRead},
    canonical_study_workbench::StudyCapture,
};
use std::sync::Arc;

/// Existing read/release owners retained independently through native completion.
/// The allocation carrier remains separate, preventing read/metadata ownership cycles.
#[derive(Clone, Debug)]
pub struct RetainedRequest(Arc<Node>);
#[derive(Debug)]
enum Node {
    Result {
        _owner: Arc<ResultRead>,
    },
    Completion {
        _owner: Arc<CompletionRead>,
    },
    Captured {
        _owner: Arc<CapturedResultRead>,
    },
    Study {
        _owner: StudyCapture,
    },
    Artifact {
        _owner: Arc<ArtifactRead>,
    },
    Selection {
        _owner: ProtectedSelection,
    },
    Joined {
        _first: RetainedRequest,
        _second: RetainedRequest,
    },
}
macro_rules! custody {
    ($ty:ty,$variant:ident) => {
        impl From<$ty> for RetainedRequest {
            fn from(owner: $ty) -> Self {
                Self(Arc::new(Node::$variant { _owner: owner }))
            }
        }
    };
}
custody!(Arc<ResultRead>, Result);
custody!(Arc<CompletionRead>, Completion);
custody!(Arc<CapturedResultRead>, Captured);
custody!(StudyCapture, Study);
custody!(Arc<ArtifactRead>, Artifact);
custody!(ProtectedSelection, Selection);
impl RetainedRequest {
    /// Compose two actual issued guards without acquiring authority or bytes.
    pub fn combine(first: impl Into<Self>, second: impl Into<Self>) -> Self {
        let first = first.into();
        let second = second.into();
        if Arc::ptr_eq(&first.0, &second.0) {
            return first;
        }
        Self(Arc::new(Node::Joined {
            _first: first,
            _second: second,
        }))
    }
}
