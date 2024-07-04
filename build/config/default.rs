#![allow(unused)]

#[macro_export]
macro_rules! define_download_url {
  ($($name:ident = $url:expr);+ $(;)?) => {
    $(paste::paste! {
      pub const [<$name _DOWNLOAD_URL>]: &str = $url;
    })+
  };

  ($($name:ident),+) => {
    define_download_url! {
      $($name = "";)+
    }
  };
}

define_download_url!(TOOLCHAIN, NINJA);
