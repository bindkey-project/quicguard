//! Description de la ligne de commande de qg-userland-v0.
use std::net::SocketAddr;
use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use ipnet::Ipv4Net;

/// QuicGuard v0 : VPN MASQUE CONNECT-IP en espace utilisateur.
#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// Les deux modes de fonctionnement du binaire.
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Lance le serveur : accepte les requêtes CONNECT-IP.
    Server(ServerArgs),
    /// Lance le client : ouvre un tunnel vers un serveur.
    Client(ClientArgs),
}

#[derive(Debug, Args)]
pub struct ServerArgs {
    /// Adresse et port UDP d'écoute.
    #[arg(long, default_value = "0.0.0.0:4433")]
    pub listen: SocketAddr,

    /// Certificat du serveur (format DER).
    #[arg(long)]
    pub cert: PathBuf,

    /// Clé privée du serveur.
    #[arg(long)]
    pub key: PathBuf,

    /// Plage d'adresses IPv4 attribuées aux clients (via ADDRESS_ASSIGN).
    #[arg(long, default_value = "10.0.0.0/24")]
    pub pool: Ipv4Net,
}

#[derive(Debug, Args)]
pub struct ClientArgs {
    /// Adresse et port UDP du serveur.
    #[arg(long)]
    pub server: SocketAddr,

    /// Nom attendu dans le certificat du serveur (SNI et :authority).
    #[arg(long, default_value = "localhost")]
    pub name: String,

    /// Certificat de confiance (CA) servant à vérifier le serveur (format DER).
    #[arg(long)]
    pub ca: PathBuf,
}
