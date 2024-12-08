#[derive(Debug)]
pub enum Error {
    Other(&'static str),

    String(String),

    Mnemonic(bip39::Error),

    Seed(crate::seed::SeedError),

    ExtractTx(bitcoin::psbt::ExtractTxError),

    Io(std::io::Error),

    Utf8(std::str::Utf8Error),

    PsbtParse(bitcoin::psbt::PsbtParseError),

    FromScript(bitcoin::address::FromScriptError),

    Miniscript(miniscript::Error),

    Descriptor(miniscript::descriptor::ConversionError),

    Parse(bitcoin::address::ParseError),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(e: std::str::Utf8Error) -> Self {
        Error::Utf8(e)
    }
}

impl From<bitcoin::psbt::PsbtParseError> for Error {
    fn from(e: bitcoin::psbt::PsbtParseError) -> Self {
        Error::PsbtParse(e)
    }
}

impl From<bitcoin::address::FromScriptError> for Error {
    fn from(e: bitcoin::address::FromScriptError) -> Self {
        Error::FromScript(e)
    }
}

impl From<bitcoin::psbt::ExtractTxError> for Error {
    fn from(e: bitcoin::psbt::ExtractTxError) -> Self {
        Error::ExtractTx(e)
    }
}

impl From<crate::seed::SeedError> for Error {
    fn from(e: crate::seed::SeedError) -> Self {
        Error::Seed(e)
    }
}

impl From<miniscript::Error> for Error {
    fn from(e: miniscript::Error) -> Self {
        Error::Miniscript(e)
    }
}

impl From<miniscript::descriptor::ConversionError> for Error {
    fn from(e: miniscript::descriptor::ConversionError) -> Self {
        Error::Descriptor(e)
    }
}

impl From<String> for Error {
    fn from(e: String) -> Self {
        Error::String(e)
    }
}

impl From<bitcoin::address::ParseError> for Error {
    fn from(e: bitcoin::address::ParseError) -> Self {
        Error::Parse(e)
    }
}

impl From<&'static str> for Error {
    fn from(e: &'static str) -> Self {
        Error::Other(e)
    }
}
