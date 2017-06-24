quick_error! {
    #[derive(Debug)]
    pub enum ConversionError {
        MissingField {
        }
        MissingValue {
        }
        InvalidValue  {
        }
    }
}
