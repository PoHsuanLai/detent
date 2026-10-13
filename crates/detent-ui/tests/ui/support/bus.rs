//! A private `dbus-daemon`: a scratch config, a scratch socket, a scratch HOME. It never reaches
//! the session bus; the guard kills the daemon by PID.

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};

pub struct PrivateBus {
    child: Child,
    dir: PathBuf,
    pub address: String,
}

impl PrivateBus {
    pub fn start(tag: &str) -> PrivateBus {
        PrivateBus::start_with_services(tag, &[])
    }

    /// A private bus that can start the daemons `names`: each has a service file whose program
    /// only writes down that it was started (`started`), then exits without taking the name.
    /// A name not listed has no service file, so asking the bus to start it fails.
    pub fn start_with_services(tag: &str, names: &[&str]) -> PrivateBus {
        let dir = PathBuf::from(super::TMP).join(format!("bus-{}-{tag}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("home")).expect("scratch dir");
        let services = dir.join("services");
        std::fs::create_dir_all(&services).expect("service dir");
        for name in names {
            std::fs::write(
                services.join(format!("{name}.service")),
                format!(
                    "[D-BUS Service]\nName={name}\nExec=/usr/bin/touch {}\n",
                    dir.join(format!("started-{name}")).display()
                ),
            )
            .expect("service file");
        }
        let config = dir.join("bus.conf");
        std::fs::write(
            &config,
            format!(
                "<busconfig><type>session</type><listen>unix:path={}</listen>\
                 <auth>EXTERNAL</auth><servicedir>{}</servicedir>\
                 <policy context=\"default\"><allow send_destination=\"*\" eavesdrop=\"true\"/>\
                 <allow eavesdrop=\"true\"/><allow own=\"*\"/></policy></busconfig>",
                dir.join("bus").display(),
                services.display()
            ),
        )
        .expect("bus config");
        // The environment is cleared: nothing of the real session reaches the daemon.
        let mut child = Command::new("dbus-daemon")
            .env_clear()
            .env("HOME", dir.join("home"))
            .env("XDG_RUNTIME_DIR", &dir)
            .arg("--config-file")
            .arg(&config)
            .arg("--nofork")
            .arg("--print-address=1")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("dbus-daemon starts");
        let stdout = child.stdout.take().expect("piped stdout");
        let mut address = String::new();
        BufReader::new(stdout)
            .read_line(&mut address)
            .expect("the daemon prints its address");
        PrivateBus {
            child,
            dir,
            address: address.trim().to_owned(),
        }
    }

    /// The daemons the bus was asked to start, sorted (those with a service file only).
    pub fn started(&self) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(&self.dir)
            .expect("the scratch dir")
            .filter_map(|entry| entry.ok()?.file_name().to_str().map(str::to_owned))
            .filter_map(|file| file.strip_prefix("started-").map(str::to_owned))
            .collect();
        names.sort();
        names
    }

    /// A connection to this bus and no other.
    pub async fn connect(&self) -> zbus::Connection {
        zbus::connection::Builder::address(self.address.as_str())
            .expect("a unix address")
            .build()
            .await
            .expect("connects to the private bus")
    }
}

impl Drop for PrivateBus {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}
