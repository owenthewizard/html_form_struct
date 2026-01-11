/// Generate a struct from an HTML form.
///
/// # Arguments
///
/// - `path`: Path to the HTML file
/// - `form`: CSS form selector
/// - `name`: Struct name
///
/// # Example
///
/// ```ignore
/// # use html_form_struct::form_struct;
///
/// #[form_struct("path/to/form.html", "form#my-form")]
/// // important: derive must come after form_struct
/// #[derive(Debug, serde::Serialize, serde::Deserialize)]
/// pub struct MyForm;
/// ```
pub use html_form_struct_macro::form_struct;
