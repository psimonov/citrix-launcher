use anyhow::{Context, Result, bail};
use regex::Regex;
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, CONTENT_TYPE, REFERER, SET_COOKIE, USER_AGENT};
use serde_json::Value;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{
    path::Path,
    process::{Command, Stdio},
    thread,
    time::Duration,
};
use url::Url;

#[cfg(windows)]
const UA: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) CitrixVdiLauncher/0.1";
#[cfg(target_os = "macos")]
const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X) CitrixVdiLauncher/0.1";
#[cfg(target_os = "linux")]
const UA: &str = "Mozilla/5.0 (X11; Linux x86_64) CitrixVdiLauncher/0.1";
#[cfg(not(any(windows, target_os = "macos", target_os = "linux")))]
const UA: &str = "CitrixVdiLauncher/0.1";
const CREDENTIAL_TYPES: &str = "none, username, domain, password, newpassword, passcode, savecredentials, textcredential, webview, nsg-epa, negotiate, nsg_push, nsg_push_otp, nf_sspr_rem, nsg-x1, nsg-setclient, nsg-eula, nsg-tlogin, nsg-fullvpn, nsg-hidden, nsg-auth-failure, nsg-auth-success, nsg-epa-success, nsg-l20n, GoBack, nf-recaptcha, ns-dialogue, nf-gw-test, nf-poll, nsg_qrcode, nsg_manageotp";
const LABEL_TYPES: &str = "none, plain, heading, information, warning, error, confirmation, image, nsg-epa, nsg-epa-failure, nsg-login-label, tlogin-failure-msg, nsg-tlogin-heading, nsg-tlogin-single-res, nsg-tlogin-multi-res, nsg-tlogin, nsg-login-heading, nsg-fullvpn, nsg-l20n, nsg-l20n-error, certauth-failure-msg, dialogue-label, nsg-change-pass-assistive-text, nsg_confirmation, nsg_kba_registration_heading, nsg_email_registration_heading, nsg_kba_validation_question, nsg_sspr_success, nf-manage-otp";

pub struct GatewaySession {
    client: Client,
    portal: Url,
    gateway_csrf: String,
    storefront_csrf: String,
    states: [[u32; 8]; 2],
    resources: String,
    pub response: String,
}

pub fn authenticate(
    base: &str,
    username: &str,
    password: &str,
    otp: &str,
) -> Result<GatewaySession> {
    let base = Url::parse(base)?;
    let client = Client::builder()
        .cookie_store(true)
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()?;
    let login = client
        .get(base.clone())
        .header(USER_AGENT, UA)
        .send()?
        .error_for_status()?;
    let login_url = login.url().clone();
    let html = login.text()?;
    let csrf = capture(
        &html,
        r#"(?i)meta name='csrf-token-value' content='([^']+)'"#,
        "CSRF",
    )?;
    let script = capture(
        &html,
        r#"(?i)<script[^>]+src=[\"']([^\"']+\.js\?[^\"']+)[\"']"#,
        "security script",
    )?;
    let script_url = login_url.join(&script)?;
    let javascript = client
        .get(script_url)
        .header(USER_AGENT, UA)
        .send()?
        .error_for_status()?
        .text()?;
    let states = parse_states(&javascript)?;

    let requirements_url = base.join("/nf/auth/getAuthenticationRequirements.do")?;
    let requirements = client
        .get(requirements_url)
        .header(USER_AGENT, UA)
        .header(ACCEPT, "application/xml, text/xml, */*; q=0.01")
        .header("X-Requested-With", "XMLHttpRequest")
        .header("X-Citrix-IsUsingHTTPS", "Yes")
        .header("X-csrftoken", &csrf)
        .send()?
        .error_for_status()?
        .text()?;
    let state = capture(
        &requirements,
        r"(?s)<StateContext\s*>([^<]+)</StateContext>",
        "StateContext",
    )?;

    let body = url::form_urlencoded::Serializer::new(String::new())
        .append_pair("login", username)
        .append_pair("passwd", password)
        .append_pair("passwd1", otp)
        .append_pair("savecredentials", "false")
        .append_pair("Logon", "Отправить")
        .append_pair("StateContext", &state)
        .finish();
    let ajax = custom_hash(
        format!("#POST#/nf/auth/doAuthentication.do#{body}").as_bytes(),
        states,
    );
    let auth_url = base.join("/nf/auth/doAuthentication.do")?;
    let response = client
        .post(auth_url)
        .header(USER_AGENT, UA)
        .header(REFERER, login_url.as_str())
        .header(ACCEPT, "application/xml, text/xml, */*; q=0.01")
        .header(
            CONTENT_TYPE,
            "application/x-www-form-urlencoded; charset=UTF-8",
        )
        .header("X-Requested-With", "XMLHttpRequest")
        .header("X-Citrix-IsUsingHTTPS", "Yes")
        .header("X-csrftoken", &csrf)
        .header("X-Ajax-Token", ajax)
        .header("X-Citrix-AM-CredentialTypes", CREDENTIAL_TYPES)
        .header("X-Citrix-AM-LabelTypes", LABEL_TYPES)
        .body(body)
        .send()?;
    let status = response.status();
    let final_url = response.url().clone();
    let mut text = response.text()?;
    if !status.is_success() {
        bail!(
            "NetScaler HTTP {status} at {final_url}: {}",
            text.chars().take(300).collect::<String>()
        );
    }
    if text.contains("<PostBack>/p/u/setClient.do</PostBack>") {
        let next_state = capture(
            &text,
            r"(?s)<StateContext\s*>([^<]+)</StateContext>",
            "setClient StateContext",
        )?;
        let body = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("nsg-setclient", "wica")
            .append_pair("StateContext", &next_state)
            .finish();
        let ajax = custom_hash(format!("#POST#/p/u/setClient.do#{body}").as_bytes(), states);
        let response = client
            .post(base.join("/p/u/setClient.do")?)
            .header(USER_AGENT, UA)
            .header(REFERER, login_url.as_str())
            .header(ACCEPT, "application/xml, text/xml, */*; q=0.01")
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header("X-Requested-With", "XMLHttpRequest")
            .header("X-Citrix-IsUsingHTTPS", "Yes")
            .header("X-csrftoken", &csrf)
            .header("X-Ajax-Token", ajax)
            .header("X-Citrix-AM-CredentialTypes", CREDENTIAL_TYPES)
            .header("X-Citrix-AM-LabelTypes", LABEL_TYPES)
            .body(body)
            .send()?;
        let status = response.status();
        text = response.text()?;
        if !status.is_success() {
            bail!(
                "setClient HTTP {status}: {}",
                text.chars().take(300).collect::<String>()
            );
        }
    }
    if text.contains("<Result>success</Result>") {
        let portal = client.get(base.clone()).header(USER_AGENT, UA).send()?;
        let portal_url = portal.url().clone();
        let portal_cookie_csrf = cookie_from_headers(portal.headers(), "CsrfToken");
        let portal_text = portal.text()?;
        let portal_meta_csrf = capture(
            &portal_text,
            r#"(?i)meta name='csrf-token-value' content='([^']+)'"#,
            "StoreFront CSRF",
        )
        .ok();
        let config_url = portal_url.join("Home/Configuration")?;
        // StoreFront creates its session and CSRF cookie on this request.  The
        // NetScaler meta token belongs to the gateway and must not be supplied
        // as StoreFront's Csrf-Token here (that causes an immediate 403).
        let config_body = "";
        let config_ajax = custom_hash(
            format!("#POST#{}#{config_body}", config_url.path()).as_bytes(),
            states,
        );
        let config_response = client
            .post(config_url.clone())
            .header(USER_AGENT, UA)
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header(ACCEPT, "application/xml, text/xml, */*; q=0.01")
            .header(REFERER, portal_url.as_str())
            .header("X-Citrix-IsUsingHTTPS", "Yes")
            .header("X-csrftoken", &csrf)
            .header("X-Ajax-Token", config_ajax)
            .body(config_body)
            .send()?;
        let config_status = config_response.status();
        let config_cookie_csrf = cookie_from_headers(config_response.headers(), "CsrfToken");
        let config_text = config_response.text()?;
        if !config_status.is_success() {
            bail!(
                "StoreFront configuration HTTP {config_status}: {}",
                config_text.chars().take(500).collect::<String>()
            );
        }
        let portal_csrf = config_cookie_csrf
            .or(portal_cookie_csrf)
            .or(portal_meta_csrf)
            .context("StoreFront did not provide a CSRF token")?;
        let resources_url = portal_url.join("Resources/List")?;
        let resources_body = "format=json&resourceDetails=Default";
        let resources_ajax = custom_hash(
            format!("#POST#{}#{resources_body}", resources_url.path()).as_bytes(),
            states,
        );
        let first_resources = client
            .post(resources_url.clone())
            .header(USER_AGENT, UA)
            .header("Csrf-Token", &portal_csrf)
            .header("X-csrftoken", &csrf)
            .header("X-Ajax-Token", &resources_ajax)
            .header("X-Citrix-IsUsingHTTPS", "Yes")
            .header("X-Requested-With", "XMLHttpRequest")
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header(REFERER, portal_url.as_str())
            .body(resources_body)
            .send()?;
        let challenge = first_resources
            .headers()
            .get("CitrixWebReceiver-Authenticate")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_owned();
        let first_text = first_resources.text()?;
        if first_text.contains("unauthorized") || challenge.contains("TokenRequired") {
            let auth_methods_url = portal_url.join("Authentication/GetAuthMethods")?;
            let methods_ajax = custom_hash(
                format!("#POST#{}#", auth_methods_url.path()).as_bytes(),
                states,
            );
            let _ = client
                .post(auth_methods_url)
                .header(USER_AGENT, UA)
                .header("Csrf-Token", &portal_csrf)
                .header("X-csrftoken", &csrf)
                .header("X-Ajax-Token", methods_ajax)
                .header("X-Citrix-IsUsingHTTPS", "Yes")
                .header(
                    CONTENT_TYPE,
                    "application/x-www-form-urlencoded; charset=UTF-8",
                )
                .body("")
                .send()?;
            let gateway_url = portal_url.join("GatewayAuth/Login")?;
            let gateway_ajax =
                custom_hash(format!("#POST#{}#", gateway_url.path()).as_bytes(), states);
            let gateway = client
                .post(gateway_url)
                .header(USER_AGENT, UA)
                .header("Csrf-Token", &portal_csrf)
                .header("X-csrftoken", &csrf)
                .header("X-Ajax-Token", gateway_ajax)
                .header("X-Citrix-IsUsingHTTPS", "Yes")
                .header(
                    CONTENT_TYPE,
                    "application/x-www-form-urlencoded; charset=UTF-8",
                )
                .body("")
                .send()?;
            let gateway_status = gateway.status();
            let gateway_text = gateway.text()?;
            if !gateway_status.is_success() || !gateway_text.contains("success") {
                bail!("StoreFront Gateway SSO failed {gateway_status}: {gateway_text}");
            }
        }
        let resources_response = client
            .post(resources_url.clone())
            .header(USER_AGENT, UA)
            .header("Csrf-Token", &portal_csrf)
            .header("X-csrftoken", &csrf)
            .header("X-Ajax-Token", resources_ajax)
            .header("X-Citrix-IsUsingHTTPS", "Yes")
            .header("X-Requested-With", "XMLHttpRequest")
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header(REFERER, portal_url.as_str())
            .body(resources_body)
            .send()?;
        let resources_status = resources_response.status();
        let resources_text = resources_response.text()?;
        if !resources_status.is_success() {
            bail!(
                "StoreFront resources HTTP {resources_status}: {}",
                resources_text.chars().take(500).collect::<String>()
            );
        }
        text = format!(
            "<!-- AUTH RESPONSE -->\n{text}\n<!-- PORTAL URL: {portal_url} -->\n{portal_text}\n<!-- CONFIG {config_status} {config_url} -->\n{config_text}\n<!-- RESOURCES {resources_status} {resources_url} -->\n{resources_text}"
        );
        Ok(GatewaySession {
            client,
            portal: portal_url,
            gateway_csrf: csrf,
            storefront_csrf: portal_csrf,
            states,
            resources: resources_text,
            response: text,
        })
    } else {
        let message = Regex::new(r"(?s)<Text>(.*?)</Text>")?
            .captures_iter(&text)
            .filter_map(|c| c.get(1).map(|m| m.as_str()))
            .last()
            .unwrap_or("authentication failed");
        bail!("NetScaler authentication was not completed: {message}");
    }
}

pub fn launch_vdi(
    session: &GatewaySession,
    names: &[String],
    citrix_path: &str,
    ica_path: &Path,
) -> Result<String> {
    let document: Value =
        serde_json::from_str(&session.resources).context("Invalid StoreFront resources JSON")?;
    let wanted: Vec<String> = names
        .iter()
        .map(|s| normalize(s))
        .filter(|s| !s.is_empty())
        .collect();
    let resources = document
        .get("resources")
        .and_then(Value::as_array)
        .context("StoreFront response has no resources array")?;
    let resource = resources
        .iter()
        .find(|r| {
            ["name", "desktophostname", "id"]
                .iter()
                .filter_map(|k| r.get(*k).and_then(Value::as_str))
                .any(|candidate| {
                    wanted
                        .iter()
                        .any(|w| normalize(candidate) == *w || normalize(candidate).contains(w))
                })
        })
        .with_context(|| {
            format!(
                "VDI not found. Available: {}",
                resources
                    .iter()
                    .filter_map(|r| r.get("name").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })?;
    let display_name = resource
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("VDI")
        .to_owned();
    let status_path = resource
        .get("launchstatusurl")
        .and_then(Value::as_str)
        .context("launchstatusurl missing")?;
    let launch_path = resource
        .get("launchurl")
        .and_then(Value::as_str)
        .context("launchurl missing")?;
    let status_url = session.portal.join(status_path)?;
    let mut ready = false;
    for _ in 0..30 {
        let ajax = custom_hash(
            format!("#POST#{}#", status_url.path()).as_bytes(),
            session.states,
        );
        let response = session
            .client
            .post(status_url.clone())
            .header(USER_AGENT, UA)
            .header(
                CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=UTF-8",
            )
            .header("Csrf-Token", &session.storefront_csrf)
            .header("X-csrftoken", &session.gateway_csrf)
            .header("X-Ajax-Token", ajax)
            .header("X-Citrix-IsUsingHTTPS", "Yes")
            .body("")
            .send()?;
        let status = response.status();
        let body = response.text()?;
        if !status.is_success() {
            bail!(
                "GetLaunchStatus HTTP {status}: {}",
                body.chars().take(500).collect::<String>()
            );
        }
        let launch: Value = serde_json::from_str(&body).context("Invalid launch status JSON")?;
        match launch.get("status").and_then(Value::as_str).unwrap_or("") {
            "success" => {
                ready = true;
                break;
            }
            "retry" => {
                let seconds = launch
                    .get("pollTimeout")
                    .and_then(Value::as_u64)
                    .unwrap_or(3)
                    .min(10);
                thread::sleep(Duration::from_secs(seconds));
            }
            "failure" => bail!("StoreFront could not prepare VDI: {body}"),
            other => bail!("Unexpected launch status '{other}': {body}"),
        }
    }
    if !ready {
        bail!("VDI was not ready before launch timeout");
    }
    let mut launch_url = session.portal.join(launch_path)?;
    launch_url
        .query_pairs_mut()
        .append_pair("CsrfToken", &session.storefront_csrf)
        .append_pair("IsUsingHttps", "Yes");
    let response = session
        .client
        .get(launch_url)
        .header(USER_AGENT, UA)
        .send()?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let bytes = response.bytes()?;
    if !status.is_success() || !bytes.starts_with(b"[") {
        bail!(
            "LaunchIca HTTP {status} ({content_type}): {}",
            String::from_utf8_lossy(&bytes[..bytes.len().min(500)])
        );
    }
    if let Some(parent) = ica_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(ica_path, &bytes)?;
    if !Path::new(citrix_path).is_file() {
        bail!("Citrix executable not found: {citrix_path}");
    }
    let mut command = Command::new(citrix_path);
    command
        .arg(ica_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    #[cfg(windows)]
    command.creation_flags(0x0000_0200); // CREATE_NEW_PROCESS_GROUP
    #[cfg(unix)]
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() < 0 {
                Err(std::io::Error::last_os_error())
            } else {
                Ok(())
            }
        });
    }
    command
        .spawn()
        .with_context(|| format!("Failed to launch Citrix: {citrix_path}"))?;
    Ok(display_name)
}

fn normalize(value: &str) -> String {
    value
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(char::to_uppercase)
        .collect()
}

fn cookie_from_headers(headers: &reqwest::header::HeaderMap, name: &str) -> Option<String> {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|h| h.to_str().ok())
        .find_map(|value| {
            value.split(';').map(str::trim).find_map(|part| {
                let (cookie_name, cookie_value) = part.split_once('=')?;
                cookie_name
                    .eq_ignore_ascii_case(name)
                    .then(|| cookie_value.to_owned())
            })
        })
}

fn capture(text: &str, pattern: &str, name: &str) -> Result<String> {
    Regex::new(pattern)?
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|x| x.as_str().to_owned())
        .with_context(|| format!("{name} not found"))
}

fn parse_states(js: &str) -> Result<[[u32; 8]; 2]> {
    let cap = Regex::new(r"hmac\.states=\[\[([^\]]+)\],\[([^\]]+)\]\]")?
        .captures(js)
        .context("hmac.states not found")?;
    let mut out = [[0u32; 8]; 2];
    for row in 0..2 {
        let nums: Vec<i64> = cap[row + 1]
            .split(',')
            .map(|x| x.trim().parse())
            .collect::<std::result::Result<_, _>>()?;
        if nums.len() != 8 {
            bail!("Invalid hmac state");
        }
        for (i, n) in nums.into_iter().enumerate() {
            out[row][i] = (n as i32) as u32;
        }
    }
    Ok(out)
}

fn custom_hash(message: &[u8], states: [[u32; 8]; 2]) -> String {
    let first = sha256_with_iv(message, states[0]);
    let second = sha256_with_iv(&first, states[1]);
    second.iter().map(|b| format!("{b:02x}")).collect()
}

fn sha256_with_iv(data: &[u8], mut h: [u32; 8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4,
        0xab1c5ed5, 0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe,
        0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f,
        0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc,
        0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b,
        0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116,
        0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    // NetScaler's bundled hmac.prepareMessage deliberately encodes
    // (message_length + one 64-byte block), rather than standard SHA-256 length.
    let bit_len = ((data.len() as u64) + 64) * 8;
    let mut p = data.to_vec();
    p.push(0x80);
    while p.len() % 64 != 56 {
        p.push(0)
    }
    p.extend_from_slice(&bit_len.to_be_bytes());
    for block in p.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, c) in block.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes(c.try_into().unwrap())
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1)
        }
        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ (!e & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2)
        }
        for (i, v) in [a, b, c, d, e, f, g, hh].iter().enumerate() {
            h[i] = h[i].wrapping_add(*v)
        }
    }
    let mut out = [0u8; 32];
    for (i, v) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&v.to_be_bytes())
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parses_states() {
        let s = "hmac.states=[[-1,2,3,4,5,6,7,8],[9,10,11,12,13,14,15,16]]";
        assert_eq!(parse_states(s).unwrap()[0][0], u32::MAX);
    }
    #[test]
    fn matches_netscaler_javascript_hash() {
        let st = [[1, 2, 3, 4, 5, 6, 7, 8], [9, 10, 11, 12, 13, 14, 15, 16]];
        let body = "login=example-user&passwd=example-password&passwd1=000000&savecredentials=false&Logon=Submit&StateContext=synthetic-state";
        assert_eq!(
            custom_hash(
                format!("#POST#/nf/auth/doAuthentication.do#{body}").as_bytes(),
                st
            ),
            "1f0634de6633bf67263ce9565e27db4928f0f70ac46f8913829ece690a2e279b"
        );
    }
}
