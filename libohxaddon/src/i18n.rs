use fluent::{FluentBundle, FluentResource, FluentMessage};
use std::collections::HashMap;
use unic_langid::LanguageIdentifier;

use std::sync::{Arc, Mutex};
use arc_swap::ArcSwap;
use std::borrow::Cow;

use log::{warn, info};

pub use fluent::FluentValue;
use std::str::FromStr;

/// Returns the translation for a given message id and arguments.
/// The translations system uses [Fluent](https://projectfluent.org/).
///
/// Clone this data type and keep a local copy where ever you require translations.
/// If the user changes the locale, translations are reloaded.
#[derive(Clone)]
pub struct Translations {
    bundle: Arc<ArcSwap<FluentBundle<FluentResource>>>,
}

impl Translations {
    /// Return a translation for a given "id"
    pub fn tr<'a, 'b>(&'b self, id: &'a str) -> Cow<'b, str> where 'a: 'b {
        self.tr_args(id, None)
    }
    /// Return a translation for a given "id". A translation string might contain placeholders.
    /// Use the optional args argument like so:
    /// let mut args = HashMap::new();
    /// args.insert("name", FluentValue::from("Rustacean"));
    pub fn tr_args<'a, 'b>(&'b self, id: &'a str, args: Option<&'b HashMap<&str, FluentValue>>) -> Cow<'b, str> where 'a: 'b {
        let bundle = self.bundle.load();
        let msg = match bundle.get_message(id) {
            None => {
                info!("Translation for '{}' does not exist", id);
                return Cow::Borrowed(id);
            }
            Some(v) => v,
        };
        let pattern = match msg.value {
            None => {
                info!("Translation '{}' has no value", id);
                return Cow::Borrowed(id);
            }
            Some(v) => v,
        };
        let mut errors = Vec::new();
        let value = bundle.format_pattern(&pattern, args, &mut errors);
        if errors.len() > 0 {
            warn!("Translation pattern for '{}' invalid", id);
            for error in errors {
                warn!("\t {}", error);
            }
        }
        Cow::Owned(value.into_owned())
    }
}

pub struct Config {
    locale: String
}

/// The translations loader will load [Fluent](https://projectfluent.org/) translation files on start up.
/// To reload translations, a restart is required.
///
/// Files are expected to have the language ID (according to https://unicode.org/reports/tr35/tr35.html#Unicode_language_identifier)
/// as base file name.
/// For example, "en-US.tr" (American English), "en_GB.tr" (British English), "es-419.tr" (Latin American Spanish).
///
/// The default language is determined by the LANG environment variable. "en_GB" is the fallback language.
pub struct TranslationLoader {
    config: Config,
    translations: Translations,
}

impl TranslationLoader {
    pub fn new() -> TranslationLoader {
        let locale = std::env::var("LANG").unwrap_or("en_GB".into());
        todo!();
        let ftl_string = "".to_owned();
        let res = FluentResource::try_new(ftl_string)
            .expect("Failed to parse an FTL string.");
        let langid_en: LanguageIdentifier = locale.parse().unwrap_or(LanguageIdentifier::from_str("en_GB").expect(""));
        let mut bundle = FluentBundle::new(&[langid_en]);
        bundle
            .add_resource(res)
            .expect("Failed to add FTL resources to the bundle.");

        TranslationLoader {
            config: Config { locale },
            translations:Translations{
                bundle: Arc::new(ArcSwap::new(Arc::new(bundle)))
            }
        }
    }
}