use super::{
    trace_columns::TraceColumns, BTreeMap, Constant, ConstantType, Constants, Identifier,
    MatrixAccess, PeriodicColumns, PublicInputs, SemanticError, TraceSegment, Variable,
    VariableType, VectorAccess, MIN_CYCLE_LENGTH,
};
use parser::ast::{PeriodicColumn, PublicInput, TraceCols};
use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) enum IdentifierType {
    /// an identifier for a constant, containing its type and value
    Constant(ConstantType),
    /// an identifier for a trace column, containing trace column information with its trace
    /// segment, its size and its offset.
    TraceColumns(TraceColumns),
    /// an identifier for a public input, containing the size of the public input array
    PublicInput(usize),
    /// an identifier for a periodic column, containing its index out of all periodic columns and
    /// its cycle length in that order.
    PeriodicColumn(usize, usize),
    /// an identifier for an integrity variable, containing its name and value
    IntegrityVariable(Variable),
}

impl Display for IdentifierType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(_) => write!(f, "Constant"),
            Self::PublicInput(_) => write!(f, "PublicInput"),
            Self::PeriodicColumn(_, _) => write!(f, "PeriodicColumn"),
            Self::TraceColumns(columns) => {
                write!(f, "TraceColumns in segment {}", columns.trace_segment())
            }
            Self::IntegrityVariable(_) => write!(f, "IntegrityVariable"),
        }
    }
}

/// SymbolTable for identifiers to track their types and information and enforce uniqueness of
/// identifiers.
#[derive(Default, Debug)]
pub(super) struct SymbolTable {
    /// Vector in which index is trace segment and value is number of columns in this segment
    segment_widths: Vec<u16>,

    /// A map of all declared identifiers from their name (the key) to their type.
    identifiers: BTreeMap<String, IdentifierType>,

    /// A vector of constants declared in the AirScript module.
    constants: Constants,

    /// A map of the Air's periodic columns using the index of the column within the declared
    /// periodic columns as the key and the vector of periodic values as the value
    periodic_columns: PeriodicColumns,

    /// A vector of public inputs with each value as a tuple of input identifier and it's array
    /// size.
    public_inputs: PublicInputs,
}

impl SymbolTable {
    /// Adds a declared identifier to the symbol table using the identifier as the key and the
    /// type the identifier represents as the value.
    ///
    /// # Errors
    /// It returns an error if the identifier already existed in the table.
    fn insert_symbol(
        &mut self,
        ident_name: &str,
        ident_type: IdentifierType,
    ) -> Result<(), SemanticError> {
        let result = self
            .identifiers
            .insert(ident_name.to_owned(), ident_type.clone());
        match result {
            Some(prev_type) => Err(SemanticError::DuplicateIdentifier(format!(
                "Cannot declare {ident_name} as a {ident_type}, since it was already defined as a {prev_type}"
            ))),
            None => Ok(()),
        }
    }

    // --- PUBLIC MUTATORS ------------------------------------------------------------------------

    /// Add all constants by their identifiers and values.
    pub(super) fn insert_constant(&mut self, constant: &Constant) -> Result<(), SemanticError> {
        let Identifier(name) = &constant.name();
        validate_constant(constant)?;
        self.insert_symbol(name, IdentifierType::Constant(constant.value().clone()))?;
        self.constants.push(constant.clone());

        Ok(())
    }

    /// Add all trace columns in the specified trace segment by their identifiers, sizes and indices.
    pub(super) fn insert_trace_columns(
        &mut self,
        trace_segment: TraceSegment,
        trace: &[TraceCols],
    ) -> Result<(), SemanticError> {
        if trace_segment >= self.segment_widths.len() as u8 {
            self.segment_widths.resize(trace_segment as usize + 1, 0);
        }

        let mut col_idx = 0;
        for trace_cols in trace {
            let trace_columns =
                TraceColumns::new(trace_segment, col_idx, trace_cols.size() as usize);
            self.insert_symbol(
                trace_cols.name(),
                IdentifierType::TraceColumns(trace_columns),
            )?;
            col_idx += trace_cols.size() as usize;
        }
        self.segment_widths[trace_segment as usize] = col_idx as u16;

        Ok(())
    }

    /// Adds all public inputs by their identifier names and array length.
    pub(super) fn insert_public_inputs(
        &mut self,
        public_inputs: &[PublicInput],
    ) -> Result<(), SemanticError> {
        for input in public_inputs.iter() {
            self.insert_symbol(input.name(), IdentifierType::PublicInput(input.size()))?;
            self.public_inputs
                .push((input.name().to_string(), input.size()));
        }

        Ok(())
    }

    /// Adds all periodic columns by their identifier names, their indices in the array of all
    /// periodic columns, and the lengths of their periodic cycles.
    pub(super) fn insert_periodic_columns(
        &mut self,
        columns: &[PeriodicColumn],
    ) -> Result<(), SemanticError> {
        for (index, column) in columns.iter().enumerate() {
            validate_cycles(column)?;
            let values = column.values().to_vec();

            self.insert_symbol(
                column.name(),
                IdentifierType::PeriodicColumn(index, values.len()),
            )?;
            self.periodic_columns.push(values);
        }

        Ok(())
    }

    /// Inserts an integrity variable into the symbol table.
    pub(super) fn insert_integrity_variable(
        &mut self,
        variable: &Variable,
    ) -> Result<(), SemanticError> {
        self.insert_symbol(
            variable.name(),
            IdentifierType::IntegrityVariable(variable.clone()),
        )?;
        Ok(())
    }

    /// Consumes this symbol table and returns the information required for declaring constants,
    /// public inputs, periodic columns and columns amount for the AIR.
    pub(super) fn into_declarations(self) -> (Vec<u16>, Constants, PublicInputs, PeriodicColumns) {
        (
            self.segment_widths,
            self.constants,
            self.public_inputs,
            self.periodic_columns,
        )
    }

    // --- ACCESSORS ------------------------------------------------------------------------------

    /// Gets the number of trace segments that were specified for this AIR.
    pub(super) fn num_trace_segments(&self) -> usize {
        self.segment_widths.len() + 1
    }

    /// Returns the type associated with the specified identifier name.
    ///
    /// # Errors
    /// Returns an error if the identifier was not in the symbol table.
    pub(super) fn get_type(&self, name: &str) -> Result<&IdentifierType, SemanticError> {
        if let Some(ident_type) = self.identifiers.get(name) {
            Ok(ident_type)
        } else {
            Err(SemanticError::InvalidIdentifier(format!(
                "Identifier {name} was not declared"
            )))
        }
    }

    pub(super) fn segment_widths(&self) -> &Vec<u16> {
        &self.segment_widths
    }

    /// Checks that the specified name and index are a valid reference to a declared public input
    /// or a vector constant and returns the symbol type. If it's not a valid reference, an error
    /// is returned.
    ///
    /// # Errors
    /// - Returns an error if the identifier is not in the symbol table.
    /// - Returns an error if the identifier is not associated with a vector access type.
    /// - Returns an error if the index is not in the declared public input array.
    /// - Returns an error if the index is greater than the vector's length.
    pub(super) fn access_vector_element(
        &self,
        vector_access: &VectorAccess,
    ) -> Result<&IdentifierType, SemanticError> {
        let symbol_type = self.get_type(vector_access.name())?;
        match symbol_type {
            IdentifierType::PublicInput(size) => {
                if vector_access.idx() < *size {
                    Ok(symbol_type)
                } else {
                    Err(SemanticError::public_inputs_out_of_bounds(
                        vector_access,
                        *size,
                    ))
                }
            }
            IdentifierType::Constant(ConstantType::Vector(vector)) => {
                validate_vector_access(vector_access, vector.len())?;
                Ok(symbol_type)
            }
            IdentifierType::IntegrityVariable(integrity_variable) => {
                if let VariableType::Vector(vector) = integrity_variable.value() {
                    validate_vector_access(vector_access, vector.len())?;
                    Ok(symbol_type)
                } else {
                    Err(SemanticError::invalid_vector_access(
                        vector_access,
                        symbol_type,
                    ))
                }
            }
            IdentifierType::TraceColumns(trace_columns) => {
                if vector_access.idx() < trace_columns.size() {
                    Ok(symbol_type)
                } else {
                    Err(SemanticError::vector_access_out_of_bounds(
                        vector_access,
                        trace_columns.size(),
                    ))
                }
            }
            _ => Err(SemanticError::invalid_vector_access(
                vector_access,
                symbol_type,
            )),
        }
    }

    /// Checks that the specified name and index are a valid reference to a matrix constant and
    /// returns the symbol type. If it's not a valid reference, an error is returned.
    ///
    /// # Errors
    /// - Returns an error if the identifier is not in the symbol table.
    /// - Returns an error if the identifier is not associated with a matrix access type.
    /// - Returns an error if the row index is greater than the matrix row length.
    /// - Returns an error if the column index is greater than the matrix column length.
    pub(super) fn access_matrix_element(
        &self,
        matrix_access: &MatrixAccess,
    ) -> Result<&IdentifierType, SemanticError> {
        let symbol_type = self.get_type(matrix_access.name())?;
        match symbol_type {
            IdentifierType::Constant(ConstantType::Matrix(matrix)) => {
                validate_matrix_access(matrix_access, matrix.len(), matrix[0].len())?;
                Ok(symbol_type)
            }
            IdentifierType::IntegrityVariable(transition_variable) => {
                if let VariableType::Matrix(matrix) = transition_variable.value() {
                    validate_matrix_access(matrix_access, matrix.len(), matrix[0].len())?;
                    Ok(symbol_type)
                } else {
                    Err(SemanticError::invalid_matrix_access(
                        matrix_access,
                        symbol_type,
                    ))
                }
            }
            _ => Err(SemanticError::invalid_matrix_access(
                matrix_access,
                symbol_type,
            )),
        }
    }
}

// HELPERS
// ================================================================================================

/// Validates the cycle length of the specified periodic column.
fn validate_cycles(column: &PeriodicColumn) -> Result<(), SemanticError> {
    let name = column.name();
    let cycle = column.values().len();

    if !cycle.is_power_of_two() {
        return Err(SemanticError::InvalidPeriodicColumn(format!(
            "cycle length must be a power of two, but was {cycle} for cycle {name}"
        )));
    }

    if cycle < MIN_CYCLE_LENGTH {
        return Err(SemanticError::InvalidPeriodicColumn(format!(
            "cycle length must be at least {MIN_CYCLE_LENGTH}, but was {cycle} for cycle {name}"
        )));
    }

    Ok(())
}

/// Checks that the declared value of a constant is valid.
fn validate_constant(constant: &Constant) -> Result<(), SemanticError> {
    match constant.value() {
        // check the number of elements in each row are same for a matrix
        ConstantType::Matrix(matrix) => {
            let row_len = matrix[0].len();
            if matrix.iter().skip(1).all(|row| row.len() == row_len) {
                Ok(())
            } else {
                Err(SemanticError::InvalidConstant(format!(
                    "The matrix value of constant {} is invalid",
                    constant.name()
                )))
            }
        }
        _ => Ok(()),
    }
}

/// Checks that the specified vector access index is valid and returns an error otherwise.
fn validate_vector_access(
    vector_access: &VectorAccess,
    vector_len: usize,
) -> Result<(), SemanticError> {
    if vector_access.idx() >= vector_len {
        return Err(SemanticError::vector_access_out_of_bounds(
            vector_access,
            vector_len,
        ));
    }
    Ok(())
}

/// Checks that the specified matrix access indices are valid and returns an error otherwise.
fn validate_matrix_access(
    matrix_access: &MatrixAccess,
    matrix_row_len: usize,
    matrix_col_len: usize,
) -> Result<(), SemanticError> {
    if matrix_access.row_idx() >= matrix_row_len {
        return Err(SemanticError::matrix_access_out_of_bounds(
            matrix_access,
            matrix_row_len,
            matrix_col_len,
        ));
    }
    if matrix_access.col_idx() >= matrix_col_len {
        return Err(SemanticError::matrix_access_out_of_bounds(
            matrix_access,
            matrix_row_len,
            matrix_col_len,
        ));
    }
    Ok(())
}