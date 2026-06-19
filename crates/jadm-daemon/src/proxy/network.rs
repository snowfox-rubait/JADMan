use anyhow::Result;
use std::process::Command;

pub struct NetworkIntercepter {
    proxy_port: u16,
    mark: u32,
}

impl NetworkIntercepter {
    pub fn new(proxy_port: u16, mark: u32) -> Self {
        Self { proxy_port, mark }
    }

    pub fn setup(&self) -> Result<()> {
        println!("Setting up network interception (redirecting to port {})", self.proxy_port);
        
        // iptables rules
        // 1. Redirect HTTP
        self.run_iptables(&[
            "-t", "nat", "-A", "OUTPUT", 
            "-p", "tcp", "--dport", "80", 
            "-m", "mark", "!", "--mark", &format!("{:#x}", self.mark), 
            "-j", "REDIRECT", "--to-port", &self.proxy_port.to_string()
        ])?;

        // 2. Redirect HTTPS
        self.run_iptables(&[
            "-t", "nat", "-A", "OUTPUT", 
            "-p", "tcp", "--dport", "443", 
            "-m", "mark", "!", "--mark", &format!("{:#x}", self.mark), 
            "-j", "REDIRECT", "--to-port", &self.proxy_port.to_string()
        ])?;

        Ok(())
    }

    pub fn teardown(&self) -> Result<()> {
        println!("Tearing down network interception");
        
        // Remove rules
        let _ = self.run_iptables(&[
            "-t", "nat", "-D", "OUTPUT", 
            "-p", "tcp", "--dport", "80", 
            "-m", "mark", "!", "--mark", &format!("{:#x}", self.mark), 
            "-j", "REDIRECT", "--to-port", &self.proxy_port.to_string()
        ]);

        let _ = self.run_iptables(&[
            "-t", "nat", "-D", "OUTPUT", 
            "-p", "tcp", "--dport", "443", 
            "-m", "mark", "!", "--mark", &format!("{:#x}", self.mark), 
            "-j", "REDIRECT", "--to-port", &self.proxy_port.to_string()
        ]);

        Ok(())
    }

    fn run_iptables(&self, args: &[&str]) -> Result<()> {
        let status = Command::new("sudo")
            .arg("iptables")
            .args(args)
            .status()?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("iptables command failed with status {}", status));
        }
        Ok(())
    }
}
