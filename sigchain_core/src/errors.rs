use super::*;

error_chain!{
    foreign_links {
        SerdeJson(serde_json::Error);
        Base64Decode(base64::DecodeError);
        Diesel(diesel::result::Error) #[cfg(feature = "db")];
        DieselMigration(diesel_migrations::RunMigrationsError) #[cfg(feature = "db")];
        DieselConnection(diesel::ConnectionError) #[cfg(feature = "db")];
        EnvVar(std::env::VarError);
        Io(std::io::Error);
        Recv(std::sync::mpsc::RecvError);
        Utf8(std::str::Utf8Error);
        SshWireDe(sshwire::serde_de::Error) #[cfg(feature = "ssh-wire")];
        Url(url::ParseError);
        Hyper(hyper::Error) #[cfg(feature = "hyper")];
        HTTP(reqwest::Error) #[cfg(feature = "reqwest")];
        Jni(jni::errors::Error) #[cfg(target_os = "android")];
    }
    links {
        Specified(specified::Error, specified::ErrorKind);
    }
}

impl From<specified::ErrorKind> for Error {
    fn from(error_kind: specified::ErrorKind) -> Self {
        specified::Error::from(error_kind).into()
    }
}

pub mod specified {
    error_chain! {
        errors {
            EmailInUse {}
            CloseInvitationsFirst {}
            AlreadyOnTeam {}
            NotAppendingToMainChain {}
            BlockExists {}
            NotAnAdmin {}
            InviteNotValid {}
            InviteLastBlockHashNotReached {}
            TeamCheckpointLastBlockHashNotReached {}
            VersionIncompatible {}
        }
    }
}
pub use self::specified::ErrorKind::*;
