/// Allows to define diagnostics more efficiently.
///
/// # Example
///
/// ```
/// use stellar_diagnostics::define_diagnostics;
/// use stellar_filesystem::location::Location;
///
/// define_diagnostics! {
///    diagnostic(error) FailedToResolveModule(
///        self,
///        module_name_location: Location,
///        module_name: String,
///        package_name_location: Location,
///        package_name: String
///    ) {
///        code { "E007" }
///        message { format!("failed to resolve the module `{}`", self.module_name) }
///        files_involved {
///            self.package_name_location.filepath,
///            self.module_name_location.filepath
///        }
///        labels {
///            primary self.module_name_location => {""},
///            secondary self.package_name_location => {
///                format!("package `{}` doesn't contain the submodule `{}`",
///                    self.package_name, self.module_name)
///            }
///        }
///        notes {}
///    }
/// }
/// ```
///
/// This macro invokations does few things:
///
/// * Creates a `FailedToResolveModule` struct with given fields.
/// * Automatically creates a constructor (`new` method) for it.
/// * Automatically implements `BuildDiagnostic` trait for a given struct.
#[macro_export]
macro_rules! define_diagnostics {
    {
        $(
            $(#[$attr:meta])*
            diagnostic($severity:ident) $name:ident (
                $self:ident
                $(,
                    $(#[$arg_attr:meta])*
                    $arg_name:ident: $arg_ty:ty
                )*
            ) {
                code { $code:expr }
                message { $message:expr }
                files_involved { $($filepath:expr),* }
                labels {
                    primary $primary_label_location:expr => { $primary_label_message:expr }
                    $(,secondary $label_location:expr => { $label_message:expr })*
                }
                notes { $($note:expr)* }
            }
        )*
    } => {
        $(
            $(#[$attr])*
            #[derive(Debug)]
            pub struct $name {
                $(
                    $(#[$arg_attr])*
                    $arg_name: $arg_ty
                ),*
            }

            impl $name {
                #[doc = concat!("A constructor for `", stringify!($name), "` generated by")]
                #[doc = concat!("`stellar_diagnostics::define_diagnostics` macro")]
                #[inline(always)]
                #[must_use]
                pub fn new($($arg_name: impl Into<$arg_ty>),*) -> Self {
                    Self { $($arg_name: $arg_name.into()),* }
                }
            }

            #[allow(clippy::unnecessary_qualification)]
            impl $crate::BuildFileDiagnostic for $name {
                #[inline(always)]
                fn build($self) -> $crate::diagnostic::Diagnostic<stellar_interner::PathID> {
                    $crate::diagnostic::Diagnostic::$severity()
                        .with_code($code.to_string())
                        .with_message($message)
                        .with_labels(vec![
                            $crate::LocationExt::to_primary_label($primary_label_location).with_message($primary_label_message)
                            $(,
                                $crate::LocationExt::to_secondary_label($label_location).with_message($label_message),
                            )*
                        ])
                        .with_notes(vec![
                            $($note.to_string()),*
                        ])
                }

                #[inline(always)]
                fn files_involved(&$self) -> Vec<stellar_interner::PathID> {
                    vec![$($filepath),*]
                }
            }
        )*
    };
}
