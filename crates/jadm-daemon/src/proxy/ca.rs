use rcgen::{Certificate, CertificateParams, KeyPair, DistinguishedName, DnType, Issuer};
use std::path::PathBuf;
use std::fs;

pub struct CertificateAuthority {
    pub cert: Certificate,
    pub key_pair: KeyPair,
}

impl CertificateAuthority {
    pub fn load_or_generate(data_dir: PathBuf) -> anyhow::Result<Self> {
        let cert_path = data_dir.join("ca.cert.pem");
        let key_path = data_dir.join("ca.key.pem");

        if cert_path.exists() && key_path.exists() {
            let key_pem = fs::read_to_string(&key_path)?;
            let key_pair = KeyPair::from_pem(&key_pem)?;
            
            // Reconstruct CA params to create the self-signed certificate
            let mut params = CertificateParams::new(vec![])?;
            let mut dn = DistinguishedName::new();
            dn.push(DnType::OrganizationName, "JADMan MITM Proxy");
            dn.push(DnType::CommonName, "JADMan Local Root CA");
            params.distinguished_name = dn;
            params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
            
            let cert = params.self_signed(&key_pair)?;
            
            return Ok(Self { cert, key_pair });
        }

        // Generate new CA
        let mut params = CertificateParams::new(vec![])?;
        let mut dn = DistinguishedName::new();
        dn.push(DnType::OrganizationName, "JADMan MITM Proxy");
        dn.push(DnType::CommonName, "JADMan Local Root CA");
        params.distinguished_name = dn;
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

        let key_pair = KeyPair::generate()?;
        let cert = params.self_signed(&key_pair)?;

        // Save to disk
        fs::write(&cert_path, cert.pem())?;
        fs::write(&key_path, key_pair.serialize_pem())?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600));
        }

        Ok(Self { cert, key_pair })
    }

    pub fn generate_leaf_cert(&self, domain: &str) -> anyhow::Result<(Certificate, KeyPair)> {
        let mut params = CertificateParams::new(vec![domain.to_string()])?;
        let mut dn = DistinguishedName::new();
        dn.push(DnType::OrganizationName, "JADMan MITM Proxy");
        dn.push(DnType::CommonName, domain);
        params.distinguished_name = dn;

        let key_pair = KeyPair::generate()?;
        
        // Reconstruct CA params to create an Issuer
        let mut ca_params = CertificateParams::new(vec![])?;
        let mut ca_dn = DistinguishedName::new();
        ca_dn.push(DnType::OrganizationName, "JADMan MITM Proxy");
        ca_dn.push(DnType::CommonName, "JADMan Local Root CA");
        ca_params.distinguished_name = ca_dn;
        ca_params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

        let issuer = Issuer::from_params(&ca_params, &self.key_pair);
        
        // Sign the leaf certificate with our CA
        let cert = params.signed_by(&key_pair, &issuer)?;
        Ok((cert, key_pair))
    }
}
