// Copyright (c) 2016-2017 Chef Software Inc. and/or applicable contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Configuration for the Supervisor.
//!
//! This module is populated from the CLI options in `main.rs`, and then passed through to the
//! [command](../command) modules. Check out the `config_from_args(..)` function there for more
//! details.
//!
//! See the [Config](struct.Config.html) struct for the specific options available.

use std::io;
use std::mem;
use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs, SocketAddr, SocketAddrV4};
use std::ops::{Deref, DerefMut};
use std::option;
use std::str::FromStr;
use std::sync::{Once, ONCE_INIT};

use hcore::package::PackageIdent;

use error::{Error, Result, SupError};
use http_gateway;
use manager::service::{Topology, UpdateStrategy};

static LOGKEY: &'static str = "CFG";

/// The Static Global Configuration.
///
/// This sets up a raw pointer, which we are going to transmute to a Box<Config>
/// with the first call to gcache().
static mut CONFIG: *const Config = 0 as *const Config;

/// Store a configuration, for later use through `gconfig()`.
///
/// MUST BE CALLED BEFORE ANY CALLS TO `gconfig()`.
pub fn gcache(config: Config) {
    static ONCE: Once = ONCE_INIT;
    unsafe {
        ONCE.call_once(|| { CONFIG = mem::transmute(Box::new(config)); });
    }
}

/// Return a reference to our cached configuration.
///
/// This is unsafe, because we are de-referencing the raw pointer stored in
/// CONFIG.
pub fn gconfig() -> &'static Config {
    unsafe { &*CONFIG }
}

/// An enum with the various CLI commands. Used to keep track of what command was called.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Start,
    ShellBash,
    ShellSh,
}

#[derive(PartialEq, Eq, Debug)]
pub struct GossipListenAddr(SocketAddr);

impl Default for GossipListenAddr {
    fn default() -> GossipListenAddr {
        GossipListenAddr(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), 9638)))
    }
}

impl Deref for GossipListenAddr {
    type Target = SocketAddr;

    fn deref(&self) -> &SocketAddr {
        &self.0
    }
}

impl DerefMut for GossipListenAddr {
    fn deref_mut(&mut self) -> &mut SocketAddr {
        &mut self.0
    }
}

impl FromStr for GossipListenAddr {
    type Err = SupError;

    fn from_str(val: &str) -> Result<Self> {
        match SocketAddr::from_str(val) {
            Ok(addr) => Ok(GossipListenAddr(addr)),
            Err(_) => {
                match IpAddr::from_str(val) {
                    Ok(ip) => {
                        let mut addr = GossipListenAddr::default();
                        addr.set_ip(ip);
                        Ok(addr)
                    }
                    Err(_) => Err(sup_error!(Error::IPFailed)),
                }
            }
        }
    }
}

impl ToSocketAddrs for GossipListenAddr {
    type Iter = option::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> io::Result<Self::Iter> {
        self.0.to_socket_addrs()
    }
}

impl FromStr for Command {
    type Err = SupError;
    fn from_str(s: &str) -> Result<Command> {
        match s {
            "bash" => Ok(Command::ShellBash),
            "sh" => Ok(Command::ShellSh),
            "start" => Ok(Command::Start),
            _ => Err(sup_error!(Error::CommandNotImplemented)),
        }
    }
}

// We provide a default command primarily so the Config struct can have reasonable defaults.
impl Default for Command {
    fn default() -> Command {
        Command::Start
    }
}

/// Holds our configuration options.
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Config {
    pub http_listen_addr: http_gateway::ListenAddr,
    pub gossip_listen: GossipListenAddr,
    command: Command,
    package: PackageIdent,
    local_artifact: Option<String>,
    url: String,
    topology: Topology,
    group: String,
    bind: Vec<String>,
    gossip_peer: Vec<String>,
    gossip_permanent: bool,
    update_strategy: UpdateStrategy,
    organization: Option<String>,
    ring: Option<String>,
    config_from: Option<String>,
}

impl Config {
    /// Create a default `Config`
    pub fn new() -> Config {
        Config::default()
    }

    /// Set the config file from directory
    pub fn set_config_from(&mut self, config_from: Option<String>) -> &mut Config {
        self.config_from = config_from;
        self
    }

    /// Return the config file from directory
    pub fn config_from(&self) -> Option<&String> {
        self.config_from.as_ref()
    }

    pub fn set_update_strategy(&mut self, strat: UpdateStrategy) -> &mut Config {
        self.update_strategy = strat;
        self
    }

    /// Return the command we used
    pub fn update_strategy(&self) -> UpdateStrategy {
        self.update_strategy
    }

    /// Set the `Command` we used
    pub fn set_command(&mut self, command: Command) -> &mut Config {
        self.command = command;
        self
    }

    /// Return the command we used
    pub fn command(&self) -> Command {
        self.command
    }

    /// Set the group
    pub fn set_group(&mut self, group: String) -> &mut Config {
        self.group = group;
        self
    }

    /// Return the group
    pub fn group(&self) -> &str {
        &self.group
    }

    /// Set the bindings
    pub fn set_bind(&mut self, bind: Vec<String>) -> &mut Config {
        self.bind = bind;
        self
    }

    /// Return the bindings
    pub fn bind(&self) -> Vec<String> {
        self.bind.clone()
    }

    /// Set the url
    pub fn set_url(&mut self, url: String) -> &mut Config {
        self.url = url;
        self
    }

    /// Return the url
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Set the topology
    pub fn set_topology(&mut self, topology: Topology) -> &mut Config {
        self.topology = topology;
        self
    }

    /// Return the topology
    pub fn topology(&self) -> Topology {
        self.topology
    }

    pub fn gossip_listen(&self) -> &GossipListenAddr {
        &self.gossip_listen
    }

    pub fn set_gossip_listen(&mut self, gossip_listen: GossipListenAddr) -> &mut Config {
        self.gossip_listen = gossip_listen;
        self
    }

    pub fn http_listen_addr(&self) -> &SocketAddr {
        &self.http_listen_addr
    }

    pub fn set_http_listen_ip(&mut self, ip: IpAddr) -> &mut Config {
        self.http_listen_addr.set_ip(ip);
        self
    }

    pub fn set_http_listen_port(&mut self, port: u16) -> &mut Config {
        self.http_listen_addr.set_port(port);
        self
    }

    pub fn gossip_permanent(&self) -> bool {
        self.gossip_permanent
    }

    pub fn set_gossip_permanent(&mut self, p: bool) -> &mut Config {
        self.gossip_permanent = p;
        self
    }

    pub fn gossip_peer(&self) -> &[String] {
        &self.gossip_peer
    }

    pub fn set_gossip_peer(&mut self, mut gp: Vec<String>) -> &mut Config {
        for p in gp.iter_mut() {
            if p.find(':').is_none() {
                p.push_str(&format!(":{}", 9638));
            }
        }
        self.gossip_peer = gp;
        self
    }

    pub fn set_package(&mut self, ident: PackageIdent) -> &mut Config {
        self.package = ident;
        self
    }

    pub fn package(&self) -> &PackageIdent {
        &self.package
    }

    pub fn set_local_artifact(&mut self, artifact: String) -> &mut Config {
        self.local_artifact = Some(artifact);
        self
    }

    pub fn local_artifact(&self) -> Option<&str> {
        self.local_artifact.as_ref().map(String::as_ref)
    }

    pub fn set_organization(&mut self, org: String) -> &mut Config {
        self.organization = Some(org);
        self
    }

    pub fn organization(&self) -> Option<&str> {
        self.organization.as_ref().map(|v| &**v)
    }

    /// Set the ring name
    pub fn set_ring(&mut self, ring: String) -> &mut Config {
        self.ring = Some(ring);
        self
    }

    /// Return the ring name
    pub fn ring(&self) -> Option<&str> {
        self.ring.as_ref().map(|v| &**v)
    }
}

#[cfg(test)]
mod tests {
    use manager::service::Topology;
    use super::{Config, Command};

    #[test]
    fn new() {
        let c = Config::new();
        assert_eq!(c.topology(), Topology::Standalone);
    }

    #[test]
    fn command() {
        let mut c = Config::new();
        c.set_command(Command::Start);
        assert_eq!(c.command(), Command::Start);
    }

    #[test]
    fn url() {
        let mut c = Config::new();
        c.set_url(String::from("http://foolio.com"));
        assert_eq!(c.url(), "http://foolio.com");
    }

    #[test]
    fn topology() {
        let mut c = Config::new();
        c.set_topology(Topology::Leader);
        assert_eq!(c.topology(), Topology::Leader);
    }
}
