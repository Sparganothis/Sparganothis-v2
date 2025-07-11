use serde::{Deserialize, Serialize};

use crate::user_identity::NodeIdentity;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, PartialOrd)]
pub struct ServerInfo {
    pub server_version: i64,
    pub server_name: String,
}

pub trait ApiMethod {
    const NAME: &'static str;
    type Arg: Clone + std::fmt::Debug + Serialize + for<'a> Deserialize<'a>;
    type Ret: std::fmt::Debug + Serialize + for<'a> Deserialize<'a>;
}

// pub struct ApiMethodInfoStatic {
//     name: &'static str,
//     arg: &'static str,
//     ret: &'static str,
// }
// #[derive(Debug, Serialize, Deserialize, PartialEq, PartialOrd, Clone)]
// pub struct ApiMethodInfo {
//     pub name: String,
//     pub arg: String,
//     pub ret: String,
// }

// impl ApiMethodInfoStatic {
//     pub const fn new(
//         name: &'static str,
//         arg: &'static str,
//         ret: &'static str,
//     ) -> ApiMethodInfoStatic {
//         ApiMethodInfoStatic {
//             name: name,
//             arg: arg,
//             ret: ret,
//         }
//     }
//     pub fn to_dynamic(&self) -> ApiMethodInfo {
//         ApiMethodInfo {
//             name: self.name.to_string(),
//             arg: self.arg.to_string(),
//             ret: self.ret.to_string(),
//         }
//     }
// }

// pub struct LoginApiMethod;
// #[async_trait]
// impl ApiMethod for LoginApiMethod {
//     const NAME: &str = "LoginApiMethod";
//     type Arg = ();
//     type Ret = ();
// }

#[macro_export]
macro_rules! declare_api_method {
    ($name:tt, $arg:ty, $ret:ty) => { paste::paste! {
        pub struct $name;
        impl $crate::server_chat_api::api_method_macros::ApiMethod for $name {
            const NAME: &str = stringify!($name);
            type Arg = $arg;
            type Ret = $ret;
        }
        // inventory::submit!{
        //     $crate::server_chat_api::api_method_macros::ApiMethodInfoStatic::new(stringify!($name), stringify!($arg), stringify!($ret))
        // }
    } }
}

pub struct ApiMethodImpl {
    pub name: &'static str,
    pub func: fn(
        NodeIdentity,
        Vec<u8>,
    ) -> std::pin::Pin<
        Box<dyn futures::Future<Output = Result<Vec<u8>, String>> + Send>,
    >,
}
// inventory::collect!(ApiMethodImpl);
// pub fn inventory_get_implementation_by_name(
//     name: &str,
// ) -> anyhow::Result<&'static ApiMethodImpl> {
//     for x in &INVENTORY_FUNCTIONS_IMPL {
//         if x.name == name {
//             return Ok(x);
//         }
//     }
//     anyhow::bail!("method not found!")
// }

#[macro_export]
macro_rules! impl_api_method {
    ($name: tt, $func_name: tt) => { $crate::paste::paste! {
        #[allow(non_snake_case)]
        async fn [< __ $name _wrapper1>] (from: $crate::user_identity::NodeIdentity, arg: Vec<u8>) -> anyhow::Result<Vec<u8>> {
                use $crate::server_chat_api::api_method_macros::ApiMethod;
                type Arg = <$name as ApiMethod>::Arg;
                type Ret = <$name as ApiMethod>::Ret;
                let arg: Arg = $crate::postcard::from_bytes(&arg)?;
                let ret: Ret = $func_name(from, arg).await?;
                let ret = $crate::postcard::to_stdvec(&ret)?;
                Ok(ret)
        }
        #[allow(non_snake_case)]
        async fn [< __ $name _wrapper2>] (from: $crate::user_identity::NodeIdentity, arg: Vec<u8>) -> Result<Vec<u8>, String> {
            let ret = [< __ $name _wrapper1>](from, arg).await.map_err(|e| format!("api method error: {e:#?}"));
            ret
        }
        #[allow(non_snake_case)]
        pub fn [< __ $name _wrapper3>] (from: $crate::user_identity::NodeIdentity, arg: Vec<u8>) -> std::pin::Pin<Box<dyn futures::Future<Output=Result<Vec<u8>, String>>+Send>> {
            let future = [< __ $name _wrapper2>](from, arg);
            use futures::FutureExt;
            future.boxed()
        }

        // $crate::inventory::submit!{
        //     $crate::server_chat_api::api_method_macros::ApiMethodImpl {
        //         name: stringify!($name),
        //         func: [< __ $name _wrapper3>]
        //     }
        // }
    } }
}

#[macro_export]
macro_rules! api_wrapper_fn {
    ($name: tt) => {
        $crate::paste::paste! {
             [< __ $name _wrapper3>]
        }
    };
}

// pub const INVENTORY_FUNCTIONS_IMPL: [ApiMethodImpl; 1] = [
//     ApiMethodImpl {
//         name: "GetReplayMatchList",
//         func: server::db::get_replay_match_list2::__GetReplayMatchList_wrapper3,
//     }
// ];

// EXAMPLE IMPLEMENTAITON OF API METHOS
// declare_api_method!(ListMethods, (), Vec<ApiMethodInfo>);
// impl_api_method!(ListMethods, _list_api_methods);
// async fn _list_api_methods(
//     _from: NodeIdentity,
//     _arg: (),
// ) -> anyhow::Result<Vec<ApiMethodInfo>> {
//     Ok(list_api_methods())
// }
// fn list_api_methods() -> Vec<ApiMethodInfo> {
//     let mut v = vec![];
//     for x in inventory::iter::<ApiMethodInfoStatic> {
//         v.push(x.to_dynamic());
//     }
//     v
// }
