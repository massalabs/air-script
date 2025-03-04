# Keywords

AirScript defines the following keywords:

- `boundary_constraints`: used to declare the source section where the [boundary constraints are described](./constraints.md#boundary_constraints).
  - `first`: used to access the value of a trace column / bus at the first row of the trace. _It may only be used when defining boundary constraints._
  - `last`: used to access the value of a trace column / bus at the last row of the trace. _It may only be used when defining boundary constraints._
- `case`: used to declare arms of [conditional constraints](./convenience.md#conditional-constraints).
- `const`: used to declare [constants](./declarations.md#constant-constant).
- `def`: used to [define the name](./organization.md#root-module) of a root AirScript module.
- `enf`: used to describe a single [constraint](./constraints.md).
  - `enf match`: used to describe [conditional constraints](./convenience.md#conditional-constraints).
- `ev`: used to declare a transition constraint [evaluator](./evaluators.md).
- `integrity_constraints`: used to declare the source section where the [integrity constraints are described](./constraints.md#integrity_constraints).
- `let`: used to declare intermediate variables in the boundary_constraints or integrity_constraints source sections.
- `mod`: used to [define a name](./organization.md#library-modules) of a library AirScript module.
- `periodic_columns`: used to declare the source section where the [periodic columns are declared](./declarations.md). _They may only be referenced when defining integrity constraints._
- `prod`: used to fold a list into a single value by multiplying all of the values in the list together.
- `public_inputs`: used to declare the source section where the [public inputs are declared](./declarations.md). _They may only be referenced when defining boundary constraints._
- `buses`: used to declare the source section where the [buses are described](./declarations.md).
  - `unit`: used to declare a multiset-based bus.
  - `mult`: used to declare a LogUp-based bus.
- `sum`: used to fold a list into a single value by summing all of the values in the list.
- `trace_columns`: used to declare the source section where the [execution trace is described](./declarations.md). _They may only be referenced when defining integrity constraints._
  - `main`: used to declare the main execution trace.
- `use`: used to [import evaluators](./organization.md#importing-evaluators) from library AirScript modules.
- `$main`: used to access columns in the main execution trace by index.
