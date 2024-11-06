use boa_engine::object::builtins::JsPromise;
use boa_engine::{
    js_string, Context, JsError, JsObject, JsResult, JsString, JsValue, Module, TryIntoJsResult,
};
use boa_interop::{IntoJsFunctionCopied, IntoJsModule};
use boa_macros::js_str;
use nix::ifaddrs::InterfaceAddress;
use nix::sys::socket::{AddressFamily, SockaddrLike, SockaddrStorage};
use reqwest::header::CONTENT_DISPOSITION;
use std::net::IpAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug)]
pub struct NetworkInterface {
    ty_: String,
    flags: Vec<String>,
    name: String,
    address: Option<IpAddr>,
    netmask: Option<IpAddr>,
    family: Option<String>,
}

fn opt_addr_to_ip_addr(addr: Option<SockaddrStorage>) -> Option<IpAddr> {
    addr.and_then(|addr| match addr.family() {
        Some(AddressFamily::Inet) => Some(IpAddr::V4(addr.as_sockaddr_in().unwrap().ip().into())),
        Some(AddressFamily::Inet6) => Some(IpAddr::V6(addr.as_sockaddr_in6().unwrap().ip().into())),
        _ => None,
    })
}

impl From<InterfaceAddress> for NetworkInterface {
    fn from(value: InterfaceAddress) -> Self {
        let flags = value.flags;
        let ty_ = if flags.contains(nix::net::if_::InterfaceFlags::IFF_LOOPBACK) {
            "Loopback"
        } else if flags.contains(nix::net::if_::InterfaceFlags::IFF_UP) {
            "Up"
        } else {
            "Down"
        };

        let name = value.interface_name.to_string();
        let address = opt_addr_to_ip_addr(value.address);
        let netmask = opt_addr_to_ip_addr(value.netmask);
        let family = if value
            .address
            .map(|a| a.family() == Some(AddressFamily::Inet))
            .unwrap_or(false)
        {
            Some("IPv4".to_string())
        } else if value
            .address
            .map(|a| a.family() == Some(AddressFamily::Inet6))
            .unwrap_or(false)
        {
            Some("IPv6".to_string())
        } else {
            None
        };

        NetworkInterface {
            ty_: ty_.to_string(),
            flags: flags
                .iter()
                .map(|f| format!("{:?}", f))
                .collect::<Vec<String>>(),
            name,
            address,
            netmask,
            family,
        }
    }
}

impl NetworkInterface {
    fn to_js_object(&self, ctx: &mut Context) -> JsResult<JsValue> {
        let obj = JsObject::with_null_proto();
        let address = self
            .address
            .map(|a| JsString::from(a.to_string().as_str()).into())
            .unwrap_or(JsValue::null());
        let netmask = self
            .netmask
            .map(|a| JsString::from(a.to_string().as_str()).into())
            .unwrap_or(JsValue::null());
        let family = self
            .family
            .as_ref()
            .map(|f| JsString::from(f.as_str()).into())
            .unwrap_or(JsValue::null());
        let flags = self
            .flags
            .iter()
            .map(|s| JsString::from(s.as_str()))
            .collect::<Vec<_>>();

        obj.set(
            js_str!("status"),
            JsString::from(self.ty_.as_str()),
            false,
            ctx,
        )?;
        obj.set(js_str!("flags"), flags.try_into_js_result(ctx)?, false, ctx)?;
        obj.set(
            js_str!("name"),
            JsString::from(self.name.as_str()),
            false,
            ctx,
        )?;
        obj.set(js_str!("address"), address, false, ctx)?;
        obj.set(js_str!("netmask"), netmask, false, ctx)?;
        obj.set(js_str!("family"), family, false, ctx)?;
        Ok(obj.into())
    }
}

fn interfaces_(ctx: &mut Context) -> JsResult<JsPromise> {
    let addrs = nix::ifaddrs::getifaddrs().map_err(JsError::from_rust)?;
    let result = addrs
        .map(|addr| NetworkInterface::from(addr))
        .filter(|i| i.family.is_some() || i.address.is_some())
        .collect::<Vec<_>>();

    result
        .into_iter()
        .map(|i| i.to_js_object(ctx))
        .collect::<Vec<_>>()
        .try_into_js_result(ctx)
        .map(|interfaces| JsPromise::resolve(interfaces, ctx))
}

fn fetch_json_(url: String, ctx: &mut Context) -> JsResult<JsPromise> {
    let result = reqwest::blocking::get(&url)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?
        .text()
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))
        .and_then(|text| {
            JsValue::from_json(
                &serde_json::Value::from_str(&text)
                    .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?,
                ctx,
            )
        });
    Ok(match result {
        Ok(v) => JsPromise::resolve(v, ctx),
        Err(e) => JsPromise::reject(e, ctx),
    })
}

fn download_(url: String, destination: Option<String>) -> JsResult<JsString> {
    let mut response = reqwest::blocking::get(&url)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    let file_name = response
        .headers()
        .get(CONTENT_DISPOSITION)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            let parts: Vec<&str> = header.split(';').collect();
            parts.iter().find_map(|part| {
                if part.trim().starts_with("filename=") {
                    Some(
                        part.trim_start_matches("filename=")
                            .trim_matches('"')
                            .to_string(),
                    )
                } else {
                    None
                }
            })
        })
        .unwrap_or_else(|| url.split('/').last().unwrap().to_string());

    let path = if let Some(dir) = destination {
        PathBuf::from(dir).join(file_name)
    } else {
        let temp_dir = tempdir::TempDir::new("golem")
            .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
        temp_dir.path().join(file_name)
    };

    std::fs::create_dir_all(path.parent().unwrap())
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;
    let mut file = std::fs::File::create(path.clone())
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    std::io::copy(&mut response, &mut file)
        .map_err(|e| JsError::from_opaque(JsString::from(e.to_string()).into()))?;

    Ok(JsString::from(path.display().to_string()))
}

fn is_online_(ctx: &mut Context) -> JsPromise {
    let is_online = ping::ping(
        [1, 1, 1, 1].into(),
        Some(Duration::from_secs(1)),
        None,
        None,
        None,
        None,
    )
    .is_ok();
    JsPromise::resolve(is_online, ctx)
}

pub fn create_module(context: &mut Context) -> JsResult<(JsString, Module)> {
    Ok((
        js_string!("net"),
        [
            (
                js_string!("interfaces"),
                interfaces_.into_js_function_copied(context),
            ),
            (
                js_string!("isOnline"),
                is_online_.into_js_function_copied(context),
            ),
            (
                js_string!("fetchJson"),
                fetch_json_.into_js_function_copied(context),
            ),
            (
                js_string!("download"),
                download_.into_js_function_copied(context),
            ),
        ]
        .into_js_module(context),
    ))
}
