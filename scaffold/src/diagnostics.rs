use errors::TlError;

pub struct ScaffoldDiagnostics {
    errors: Vec<TlError>,
}

impl ScaffoldDiagnostics {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    pub fn add_error(&mut self, error: TlError) {
        self.errors.push(error);
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Default for ScaffoldDiagnostics {
    fn default() -> Self {
        Self::new()
    }
}