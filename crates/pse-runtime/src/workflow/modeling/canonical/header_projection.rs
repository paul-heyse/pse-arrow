// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright (c) 2026 Paul Heyse
//! Selected declaration headers never copy the unrequested scientific bodies.
use super::*;
use pse_model::{
    boundary_projection::ValidateProjection as _, engineering::EngineeringDeclarationHeader,
};

impl ModelingPackage {
    /// Project only selected original declaration identities and names from authenticated ingress.
    /// The compact output carries its native allocation owner; the temporary ingress frame is released.
    /// # Errors
    /// Invalid native bounds, source/service mismatch, corrupt membership, cancellation and resource refusal propagate.
    pub async fn declaration_headers_page(
        &self,
        services: &crate::workflow::PreparationServices,
        request: &crate::workflow::ModelingDeclarationRequest,
        cancel: &crate::CancelSource,
    ) -> Result<pse_columnar::Leased<(usize, Vec<EngineeringDeclarationHeader>)>, WorkflowError>
    {
        request.validate_projection().map_err(|error| {
            WorkflowError::Typed(pse_model::diagnostic::DiagnosticCause::new(error))
        })?;
        self.check_service_bindings(services)?;
        let activity = services.admit_activity()?;
        activity
            .scope(async {
                let page = self.ordered_declaration_projection(
                    services,
                    request.offset,
                    request.limit,
                    None,
                    cancel,
                    "modeling:selected-declaration-header",
                    // Account the compact native header, generated message and complete encoded buffer.
                    |row| {
                        row.name
                            .len()
                            .saturating_mul(8)
                            .saturating_add(size_of::<EngineeringDeclarationHeader>() * 2)
                    },
                    |row| EngineeringDeclarationHeader {
                        declaration: row.declaration_id.as_id(),
                        name: row.name.clone(),
                    },
                )?;
                Ok(pse_columnar::Leased::new(
                    Arc::new((page.total, page.rows)),
                    Arc::new(page.leases),
                ))
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn compact_headers_preserve_order_and_release_their_allocation() {
        let runtime = crate::workflow::tests::runtime();
        let rows = super::super::tests::parse("package p {param a:Scalar=1;param b:Scalar=2;}");
        let package = runtime
            .modeling_package(rows.clone(), crate::workflow::tests::physical())
            .await
            .unwrap();
        let services = runtime.preparation_services();
        let baseline = services.pool().reserved();
        let cancel = crate::CancelSource::new();
        let page = package
            .declaration_headers_page(
                &services,
                &crate::workflow::ModelingDeclarationRequest {
                    offset: 1,
                    limit: 1,
                },
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(page.0, rows.len());
        assert_eq!(page.1.len(), 1);
        assert_eq!(page.1[0].declaration, rows[1].declaration_id.as_id());
        assert_eq!(page.1[0].name, rows[1].name);
        assert!(services.pool().reserved() > baseline);
        assert!(package.revision.delta_products.get().is_none());
        drop(page);
        assert_eq!(services.pool().reserved(), baseline);
        let empty = package
            .declaration_headers_page(
                &services,
                &crate::workflow::ModelingDeclarationRequest {
                    offset: rows.len(),
                    limit: 1,
                },
                &cancel,
            )
            .await
            .unwrap();
        assert_eq!(empty.0, rows.len());
        assert!(empty.1.is_empty());
        drop(empty);
        assert_eq!(services.pool().reserved(), baseline);
        cancel.cancel();
        assert!(
            package
                .declaration_headers_page(
                    &services,
                    &crate::workflow::ModelingDeclarationRequest {
                        offset: 0,
                        limit: 1
                    },
                    &cancel
                )
                .await
                .is_err()
        );
        assert_eq!(services.pool().reserved(), baseline);
        assert!(
            package
                .declaration_headers_page(
                    &services,
                    &crate::workflow::ModelingDeclarationRequest {
                        offset: 0,
                        limit: 0
                    },
                    &crate::CancelSource::new()
                )
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn producer_projects_only_selected_original_rows_before_copying() {
        let runtime = crate::workflow::tests::runtime();
        let rows = super::super::tests::parse(
            "package p {param a:Scalar=1;param b:Scalar=2;param c:Scalar=3;}",
        );
        let package = runtime
            .modeling_package(rows.clone(), crate::workflow::tests::physical())
            .await
            .unwrap();
        let seen = std::cell::RefCell::new(Vec::new());
        let extents = std::cell::Cell::new(0usize);
        let page = package
            .ordered_declaration_projection(
                &runtime.preparation_services(),
                2,
                1,
                None,
                &crate::CancelSource::new(),
                "test:selected-header-projection",
                |_| {
                    extents.set(extents.get() + 1);
                    0
                },
                |row| {
                    seen.borrow_mut().push(row.declaration_id);
                    row.declaration_id
                },
            )
            .unwrap();
        assert_eq!(page.total, rows.len());
        assert_eq!(page.rows, vec![rows[2].declaration_id]);
        assert_eq!(*seen.borrow(), vec![rows[2].declaration_id]);
        assert_eq!(
            extents.get(),
            1,
            "unselected bodies never reach extent/copy functions"
        );
        assert!(package.revision.delta_products.get().is_none());
    }
}
