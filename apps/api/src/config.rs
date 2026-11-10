#[derive(Clone, Debug)]
pub struct Config { pub dev_auth: bool, pub origin: String, pub bind: String, pub default_user: String, pub max_volunteer_minutes:i32 }
impl Config {
 pub fn validate(environment: &str, enabled: bool, origin: &str, allow_lan_dev_auth: bool) -> Result<(), String> {
  if environment != "development" && enabled { return Err("DEV_AUTH_ENABLED is forbidden outside development".into()); }
  if !enabled { return Err("No implemented production auth provider; myFWA OIDC adapter is not yet available".into()); }
  let loopback_origin=["http://localhost:","http://127.0.0.1:","https://localhost:","https://127.0.0.1:"].iter().any(|prefix| origin.starts_with(prefix) && origin[prefix.len()..].parse::<u16>().is_ok());
  let lan_origin=origin.strip_prefix("http://").and_then(|value| value.rsplit_once(':')).and_then(|(host,port)| host.parse::<std::net::Ipv4Addr>().ok().filter(|address| address.is_private()).zip(port.parse::<u16>().ok())).is_some();
  if !loopback_origin && !(allow_lan_dev_auth && lan_origin) { return Err("Development origin must be a loopback origin with an explicit port, unless DEV_AUTH_ALLOW_LAN=true specifies a private IPv4 origin".into()); }
  Ok(())
 }
 pub fn from_env() -> Result<Self, String> {
  let environment=std::env::var("APP_ENV").unwrap_or_else(|_|"production".into());
  let dev_auth=std::env::var("DEV_AUTH_ENABLED").as_deref()==Ok("true");
  let origin=std::env::var("APP_PUBLIC_ORIGIN").unwrap_or_else(|_|"http://localhost:5173".into());
  let allow_lan_dev_auth=std::env::var("DEV_AUTH_ALLOW_LAN").map(|value| value=="true").unwrap_or(false);
  Self::validate(&environment,dev_auth,&origin,allow_lan_dev_auth)?;
  let max_volunteer_minutes=std::env::var("MAX_VOLUNTEER_MINUTES").unwrap_or_else(|_|"960".into()).parse::<i32>().map_err(|_|"MAX_VOLUNTEER_MINUTES must be an integer".to_string())?;
  if !(1..=960).contains(&max_volunteer_minutes){return Err("MAX_VOLUNTEER_MINUTES must be between 1 and 960".into());}
  Ok(Self {dev_auth,origin,max_volunteer_minutes,bind:std::env::var("API_BIND").unwrap_or_else(|_|"127.0.0.1:3000".into()),default_user:std::env::var("DEV_AUTH_DEFAULT_USER").unwrap_or_else(|_|"dev.hr@purrsonnel.local".into())})
 }
}
