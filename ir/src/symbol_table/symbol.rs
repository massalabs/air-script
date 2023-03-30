use super::{
    symbol_access::ValidateAccess, AccessType, ConstantType, ConstantValue, Identifier,
    MatrixAccess, SemanticError, SymbolType, TraceAccess, TraceColumns, Value, VectorAccess,
    CURRENT_ROW,
};

/// Symbol information for a constant, variable, trace column, periodic column, or public input.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) struct Symbol {
    name: String,
    symbol_type: SymbolType,
}

impl Symbol {
    pub(super) fn new(name: String, symbol_type: SymbolType) -> Self {
        Self { name, symbol_type }
    }

    // --- PUBLIC ACCESSORS -----------------------------------------------------------------------

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn symbol_type(&self) -> &SymbolType {
        &self.symbol_type
    }

    pub fn get_value(&self, access_type: &AccessType) -> Result<Value, SemanticError> {
        match self.symbol_type() {
            SymbolType::Constant(constant_type) => {
                self.get_constant_value(constant_type, access_type)
            }
            SymbolType::PeriodicColumn(index, cycle_len) => {
                self.get_periodic_column_value(*index, *cycle_len, access_type)
            }
            SymbolType::PublicInput(size) => self.get_public_input_value(*size, access_type),
            SymbolType::RandomValuesBinding(offset, size) => {
                self.get_random_value(*offset, *size, access_type)
            }
            SymbolType::TraceColumns(columns) => self.get_trace_value(columns, access_type),
            SymbolType::Variable(_) => {
                unreachable!("Variable values cannot be accessed directly, since they reference expressions which must be added to the graph");
            }
        }
    }

    // --- VALUE ACCESS HELPERS -------------------------------------------------------------------

    fn get_constant_value(
        &self,
        constant_type: &ConstantType,
        access_type: &AccessType,
    ) -> Result<Value, SemanticError> {
        constant_type.validate(self.name(), access_type)?;

        let name = self.name().to_string();
        match access_type {
            AccessType::Default => Ok(Value::Constant(ConstantValue::Scalar(name))),
            AccessType::Vector(idx) => {
                let access = VectorAccess::new(Identifier(name), *idx);
                Ok(Value::Constant(ConstantValue::Vector(access)))
            }
            AccessType::Matrix(row_idx, col_idx) => {
                let access = MatrixAccess::new(Identifier(name), *row_idx, *col_idx);
                Ok(Value::Constant(ConstantValue::Matrix(access)))
            }
        }
    }

    fn get_periodic_column_value(
        &self,
        index: usize,
        cycle_len: usize,
        access_type: &AccessType,
    ) -> Result<Value, SemanticError> {
        match access_type {
            AccessType::Default => Ok(Value::PeriodicColumn(index, cycle_len)),
            _ => Err(SemanticError::invalid_periodic_column_access_type(
                self.name(),
            )),
        }
    }

    fn get_public_input_value(
        &self,
        size: usize,
        access_type: &AccessType,
    ) -> Result<Value, SemanticError> {
        match access_type {
            AccessType::Vector(index) => {
                if *index >= size {
                    return Err(SemanticError::vector_access_out_of_bounds(
                        self.name(),
                        *index,
                        size,
                    ));
                }
                return Ok(Value::PublicInput(self.name().to_string(), *index));
            }
            _ => return Err(SemanticError::invalid_public_input_access_type(self.name())),
        }
    }

    fn get_random_value(
        &self,
        binding_offset: usize,
        binding_size: usize,
        access_type: &AccessType,
    ) -> Result<Value, SemanticError> {
        match access_type {
            AccessType::Default => {
                if binding_size != 1 {
                    return Err(SemanticError::invalid_random_value_binding_access(
                        self.name(),
                    ));
                }
                Ok(Value::RandomValue(binding_offset))
            }
            AccessType::Vector(idx) => {
                if *idx >= binding_size {
                    return Err(SemanticError::vector_access_out_of_bounds(
                        self.name(),
                        *idx,
                        binding_size,
                    ));
                }

                let offset = binding_offset + idx;
                Ok(Value::RandomValue(offset))
            }
            _ => Err(SemanticError::invalid_random_value_access_type(
                self.name(),
                access_type,
            )),
        }
    }

    fn get_trace_value(
        &self,
        columns: &TraceColumns,
        access_type: &AccessType,
    ) -> Result<Value, SemanticError> {
        // symbol accesses at rows other than the first are identified by the parser as
        // [NamedTraceAccess] and handled differently, so this case will only occur for
        // trace column accesses at the current row.
        // TODO: can we handle this differently so it's more explicit & get rid of this comment?
        let row_offset = CURRENT_ROW;
        match access_type {
            AccessType::Default => {
                if columns.size() != 1 {
                    return Err(SemanticError::invalid_trace_binding_access(self.name()));
                }
                let trace_segment = columns.trace_segment();
                let trace_access = TraceAccess::new(trace_segment, columns.offset(), 1, row_offset);
                Ok(Value::TraceElement(trace_access))
            }
            AccessType::Vector(idx) => {
                if *idx >= columns.size() {
                    return Err(SemanticError::vector_access_out_of_bounds(
                        self.name(),
                        *idx,
                        columns.size(),
                    ));
                }

                let trace_segment = columns.trace_segment();
                let trace_access =
                    TraceAccess::new(trace_segment, columns.offset() + idx, 1, row_offset);
                Ok(Value::TraceElement(trace_access))
            }
            _ => Err(SemanticError::invalid_trace_access_type(
                self.name(),
                access_type,
            )),
        }
    }
}