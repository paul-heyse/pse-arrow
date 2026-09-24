# Symptoms and discriminating evidence

Each route exposes alternatives and evidence limits.

## filter

- [Distinguish local filtering and SDK sampling](../capabilities/tracing.filtering.md): Different layers can accept different telemetry. A local formatted span is evidence for that output branch, not proof that an SDK exported it.
- [Match macro targets and levels](../capabilities/tracing.targets.md): The default macro target comes from module_path!() at the call site; explicit target and macro level change filter selection.

## field-schema

- [Declare and record custom span fields](../capabilities/tracing.fields.md): A custom field value and its declaration are separate inputs. Tracing fixes a span's field set at creation.

## preview

- [Preview rows and choose formatting](../capabilities/tracing.preview.md): A positive preview limit enables the recorder. An omitted formatter selects the default pretty formatter; a custom callback changes presentation.

## metrics

- [Record native plan metrics](../capabilities/tracing.metrics.md): The recorder reads native plan metrics, aggregates by name, and records dynamic datafusion.metrics.* fields when its recorder is dropped.

## lifecycle

- [Follow execution and recorder lifetimes](../capabilities/tracing.lifecycle.md): The implementation groups active execution streams and holds recording state through stream ownership. Plan handles, stream handles and exported spans have distinct lifetimes.

## versions

- [Match dependency and feature profiles](../capabilities/tracing.compatibility.md): The historical release table records declared combinations. Resolved dependencies, features, successful compilation and delivery are different evidence levels.
