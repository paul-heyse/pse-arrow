# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Build provenance of the compiled extension (blueprint §21, plan §5).

This is the one module allowed to import :mod:`pse._native` (ast-grep rule
``no-direct-native-import``): every other consumer goes through
the typed build and immutable inspection adapters, so the extension's surface
has a single typed gate.

The extension embeds the exact lockfiles. Its Rust build producer records their
external SHA256 checksums, and the generated BuildInfo document carries those
captured values alongside compiler, profile and source provenance.
"""

from __future__ import annotations

from typing import TYPE_CHECKING

from pse import _native, contracts

if TYPE_CHECKING:
    from pse.contracts.documents import BuildInfo

# The same native import gateway carries the immutable inspection capabilities.
semantic_id_from_hex = _native.semantic_id_from_hex
semantic_id_to_hex = _native.semantic_id_to_hex
content_hash_from_prefixed = _native.content_hash_from_prefixed
content_hash_to_prefixed = _native.content_hash_to_prefixed

EngineSettings = _native.EngineSettings
CacheSettings = _native.CacheSettings
InspectionError = _native.InspectionError
DiagnosticReport = _native.DiagnosticReport
_NativePublication = _native.Publication
_NativeTableStream = _native.TableStream
_NativePhysicalContext = _native.NativePhysicalContext
_NativeRuntime = _native.NativeRuntime
OperationalStore = _native.OperationalStore
_NativeModelingNativeAnalysis = _native.NativeModelingNativeAnalysis
_NativeModelingNonlinearExplanation = _native.NativeModelingNonlinearExplanation
_NativeModelingElasticAttempt = _native.NativeModelingElasticAttempt
ModelingLimits = _native.ModelingLimits
ModelingDiagnosticSettings = _native.ModelingDiagnosticSettings
_NativeModelingDiagnostics = _native.NativeModelingDiagnostics
_NativeModelingDiagnosticSamples = _native.NativeModelingDiagnosticSamples
_NativeModelingTrajectory = _native.NativeModelingTrajectory
_NativeModelingPackage = _native.NativeModelingPackage
_NativeModelingKnowledge = _native.NativeModelingKnowledge
_NativeModelingConformance = _native.NativeModelingConformance
_NativeModelingResult = _native.NativeModelingResult
_NativeModelingInitialization = _native.NativeModelingInitialization
_NativeModelingInitializationAttempt = _native.NativeModelingInitializationAttempt
_NativeStudyReport = _native.NativeStudyReport
_NativeStart = _native.NativeStart
_NativePreparedStrategy = _native.NativePreparedStrategy
_NativePreparedFlow = _native.NativePreparedFlow
_NativeStrategyResult = _native.NativeStrategyResult
NativeAttempt = _native.NativeAttempt
_NativePreparedOperation = _native.NativePreparedOperation
SimulationSettings = _native.SimulationSettings
_NativeRunHandle = _native.NativeRunHandle
_NativeStudyHandle = _native.NativeStudyHandle
_NativeRunResult = _native.NativeRunResult
_NativePublicationAttempt = _native.NativePublicationAttempt
NativeStrategyAttempt = _native.NativeStrategyAttempt
_NativeProgressStream = _native.NativeProgressStream


def _open_export(location: str, settings: EngineSettings) -> _NativePublication:
    return _native.open_export(location, settings)


def build_info() -> BuildInfo:
    """Observe the compiled extension's Rust-owned provenance document."""
    return _native.build_info()


def native_version() -> str:
    """Return ``pse._native.__version__``, the version the extension was built at.

    Returns:
        The extension's own version string.
    """
    return _native.__version__


def _check_native_compatibility(package_version: str) -> None:
    """Check the compiled version and generated declaration identity without a scan."""
    if package_version != native_version():
        message = "pse package and native extension versions differ; run just py-sync"
        raise ImportError(message)
    if _native.registry_fingerprint() != contracts.REGISTRY_FINGERPRINT:
        message = (
            "pse generated contracts and native registry differ; "
            "regenerate and run just py-sync"
        )
        raise ImportError(message)
