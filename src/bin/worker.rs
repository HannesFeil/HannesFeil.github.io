//! Webworker application module

use website::theme::LoadSyntaxTheme;
use yew_agent::Registrable;

/// The entry point for the webworker
/// 
/// Registers the [`LoadSyntaxTheme`] webworker
pub fn main() {
    LoadSyntaxTheme::registrar().register();
}
