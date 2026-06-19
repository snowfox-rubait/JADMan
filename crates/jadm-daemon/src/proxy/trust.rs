use anyhow::Result;
use std::process::Command;
use std::path::Path;
use std::fs;

pub struct TrustManager {
    ca_cert_path: std::path::PathBuf,
}

impl TrustManager {
    pub fn new(ca_cert_path: std::path::PathBuf) -> Self {
        Self { ca_cert_path }
    }

    pub fn install(&self) -> Result<()> {
        println!("Installing Root CA into trust stores...");

        // 1. System-wide trust store
        self.install_system_wide()?;

        // 2. Chrome/NSS trust store (~/.pki/nssdb)
        self.install_nssdb()?;

        Ok(())
    }

    fn install_system_wide(&self) -> Result<()> {
        // Debian/Ubuntu style
        if Path::new("/usr/local/share/ca-certificates/").exists() {
            println!("Detected Debian/Ubuntu style trust store");
            let dest = Path::new("/usr/local/share/ca-certificates/jadm-ca.crt");
            self.run_sudo(&["cp", self.ca_cert_path.to_str().unwrap(), &dest.to_string_lossy()])?;
            self.run_sudo(&["update-ca-certificates"])?;
        } 
        // Fedora/Arch/CentOS style
        else if Path::new("/etc/pki/ca-trust/source/anchors/").exists() {
            println!("Detected Fedora/Arch/CentOS style trust store");
            let dest = Path::new("/etc/pki/ca-trust/source/anchors/jadm-ca.crt");
            self.run_sudo(&["cp", self.ca_cert_path.to_str().unwrap(), &dest.to_string_lossy()])?;
            self.run_sudo(&["update-ca-trust"])?;
        }
        else {
            eprintln!("Warning: Could not determine system trust store path. Skipping system-wide installation.");
        }
        Ok(())
    }

    fn install_nssdb(&self) -> Result<()> {
        let home = std::env::var("HOME")?;
        let nss_db_path = format!("{}/.pki/nssdb", home);
        
        if Path::new(&nss_db_path).exists() {
            println!("Installing into NSS DB at {}", nss_db_path);
            
            // Check if certutil is installed
            if which::which("certutil").is_ok() {
                // Delete if exists to avoid error
                let _ = Command::new("certutil")
                    .args(&["-D", "-d", &format!("sql:{}", nss_db_path), "-n", "JADMan Local Root CA"])
                    .status();

                let status = Command::new("certutil")
                    .args(&[
                        "-A", "-d", &format!("sql:{}", nss_db_path), 
                        "-t", "C,,", 
                        "-n", "JADMan Local Root CA", 
                        "-i", self.ca_cert_path.to_str().unwrap()
                    ])
                    .status()?;
                
                if !status.success() {
                    return Err(anyhow::anyhow!("certutil command failed with status {}", status));
                }
            } else {
                eprintln!("Warning: certutil not found. Skipping NSS DB installation. Chrome may still show warnings.");
            }
        }
        Ok(())
    }

    fn run_sudo(&self, args: &[&str]) -> Result<()> {
        let status = Command::new("sudo")
            .args(args)
            .status()?;
        
        if !status.success() {
            return Err(anyhow::anyhow!("sudo command failed with status {}", status));
        }
        Ok(())
    }
}
